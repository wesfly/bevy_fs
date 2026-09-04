use crate::{
    ResetMessage,
    scenery::terrain::{Coord, TerrainCacheResource, coord_to_world_pos, get_height_at_coord},
};
use avian3d::{
    collision::collider::{ColliderConstructor, ColliderConstructorHierarchy},
    dynamics::rigid_body::RigidBody,
};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    tasks::{Task, futures_lite::future},
};
use big_space::{
    floating_origins::BigSpace,
    grid::{Grid, cell::CellCoord},
};
use earcut::Earcut;

type BuildingTask = (
    Mesh,
    ColliderConstructorHierarchy,
    CellCoord,
    Vec3,
    Vec3,
    Coord,
);

#[derive(Component)]
pub struct SpawnBuilding {
    pub task: Task<Vec<BuildingTask>>,
}

#[derive(Component)]
struct Building;

#[derive(Resource)]
pub struct BuildingMaterial(pub Handle<StandardMaterial>);

impl FromWorld for BuildingMaterial {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        BuildingMaterial(materials.add(StandardMaterial::from_color(
            Color::WHITE.with_luminance(0.5),
        )))
    }
}

pub async fn spawn(grid: Grid, lat1: f32, lon1: f32, lat2: f32, lon2: f32) -> Vec<BuildingTask> {
    let buildings = kestrel_osm::buildings(kestrel_osm::CoordBounds {
        lat1,
        lon1,
        lat2,
        lon2,
    })
    .await
    .unwrap_or_default();

    let mut result_vec: Vec<BuildingTask> = Vec::with_capacity(buildings.len());

    // Reused across every building in this tile instead of constructed per-polygon: `Earcut`
    // keeps its own internal scratch buffers and its docs explicitly say to reuse one instance
    // across triangulations to avoid repeated allocation. `flat_verts`/`triangles` are just
    // scratch space around it, so they're reused the same way.
    let mut earcut = Earcut::<f64>::new();
    let mut flat_verts: Vec<[f64; 2]> = Vec::new();
    let mut triangles: Vec<u32> = Vec::new();

    for building in buildings {
        if building.nodes.is_empty() {
            continue;
        }

        let coord = Coord {
            lat: building.nodes[0].lat as f32,
            long: building.nodes[0].long as f32,
        };

        let node0_pos = coord_to_world_pos(coord).as_vec3();
        let up = node0_pos.normalize();
        let (cell_coord, cell_offset) = grid.translation_to_grid(node0_pos);

        let mut node_positions: Vec<Vec3> = building
            .nodes
            .iter()
            .map(|node| {
                coord_to_world_pos(Coord {
                    lat: node.lat as f32,
                    long: node.long as f32,
                })
                .as_vec3()
                    - node0_pos
            })
            .collect();

        // Check for duplicates
        if node_positions.len() > 1 && node_positions.first() == node_positions.last() {
            node_positions.pop();
        }

        // Check whether building can even be constructed
        let num_nodes = node_positions.len();
        if num_nodes < 3 {
            continue;
        }

        // Local tangent-plane basis at this building, perpendicular to `up`. The world is a
        // sphere, so `up` points in a different world-space direction at every building --
        // treating the raw world (x, z) components as the footprint's 2D plane (the old
        // approach) only happens to be correct near wherever up ≈ +Y. Everywhere else,
        // including the southern hemisphere, it flattens onto the wrong plane and can flip
        // the winding, sending normals downward. Projecting onto a basis built from the
        // actual local `up` makes this correct everywhere on the globe.
        let helper = if up.dot(Vec3::Y).abs() > 0.99 {
            Vec3::X
        } else {
            Vec3::Y
        };
        let e_axis = up.cross(helper).normalize();
        let n_axis = e_axis.cross(up); // already unit length: e_axis ⟂ up, both unit vectors
        let local_2d = |p: Vec3| [p.dot(e_axis) as f64, p.dot(n_axis) as f64];

        let mut area = 0.0;
        for i in 0..num_nodes {
            let j = (i + 1) % num_nodes;
            let [xi, yi] = local_2d(node_positions[i]);
            let [xj, yj] = local_2d(node_positions[j]);
            area += xi * yj - xj * yi;
        }

        if area > 0.0 {
            node_positions.reverse();
        }

        let height_offset = building.levels as f32 * up * 3.0;

        let max_triangles = num_nodes * 3;
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(max_triangles * 3);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(max_triangles * 3);
        let mut indices: Vec<u32> = Vec::with_capacity(max_triangles * 3);

        let mut push_triangle = |a: Vec3, b: Vec3, c: Vec3| {
            let normal = (b - a).cross(c - a).normalize_or_zero();
            let start = positions.len() as u32;
            positions.push(a.to_array());
            positions.push(b.to_array());
            positions.push(c.to_array());
            normals.push(normal.to_array());
            normals.push(normal.to_array());
            normals.push(normal.to_array());
            indices.extend_from_slice(&[start, start + 1, start + 2]);
        };

        // Walls
        for i in 0..num_nodes {
            let next_i = (i + 1) % num_nodes;
            let base_curr = node_positions[i];
            let base_next = node_positions[next_i];
            let top_curr = base_curr + height_offset;
            let top_next = base_next + height_offset;

            push_triangle(base_curr, base_next, top_curr);
            push_triangle(top_curr, base_next, top_next);
        }

        // Roof: triangulate the footprint in the local tangent-plane basis (not world x,z).
        // No holes in a building footprint, so hole_indices is empty -- it is NOT related to
        // the wall index buffer. `earcut`/`flat_verts`/`triangles` are the buffers hoisted
        // above the loop and reused for every building.
        flat_verts.clear();
        flat_verts.extend(node_positions.iter().map(|p| local_2d(*p)));
        earcut.earcut(flat_verts.iter().copied(), &[], &mut triangles);

        for chunk in triangles.chunks(3) {
            if chunk.len() == 3 {
                let p0 = node_positions[chunk[0] as usize] + height_offset;
                let p1 = node_positions[chunk[1] as usize] + height_offset;
                let p2 = node_positions[chunk[2] as usize] + height_offset;
                // Reversed winding (p0, p2, p1) points the normal outward/upward -- consistent
                // with the `area` winding fix above, this now holds on both hemispheres instead
                // of only near one pole.
                push_triangle(p0, p2, p1);
            }
        }

        drop(push_triangle); // end the mutable borrow of positions/normals/indices

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices));

        result_vec.push((
            mesh,
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            cell_coord,
            cell_offset,
            up,
            coord,
        ));
    }
    result_vec
}

pub fn poll(
    mut building_tasks: Query<(Entity, &mut SpawnBuilding)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    building_material: Res<BuildingMaterial>,
    big_space: Single<Entity, With<BigSpace>>,
    tile_cache: Res<TerrainCacheResource>,
    reset_messages: MessageReader<ResetMessage>,
) {
    let parent = *big_space;

    if !reset_messages.is_empty() {
        commands.entity(parent).insert(Visibility::default());
    }

    for (entity, mut task) in &mut building_tasks {
        if let Some(mesh_collider) = future::block_on(future::poll_once(&mut task.task)) {
            let bundles: Vec<_> = mesh_collider
                .into_iter()
                .map(|(mesh, collider, cell_coord, cell_offset, up, coord)| {
                    (
                        Building,
                        cell_coord,
                        Transform::from_translation(
                            cell_offset + up * get_height_at_coord(coord, 12, &tile_cache.cache),
                        ),
                        RigidBody::Static,
                        MeshMaterial3d(building_material.0.clone()),
                        Mesh3d(meshes.add(mesh)),
                        collider,
                        ChildOf(parent),
                    )
                })
                .collect();

            commands.spawn_batch(bundles);

            commands.entity(entity).remove::<SpawnBuilding>();
        }
    }
}
