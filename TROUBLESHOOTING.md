# Troubleshooting

This is a little troubleshooting guide to problems I had in the past.

## Broken git lfs on Codeberg

`Error downloading object: <some object>: Smudge error: Error downloading <some file path>: [<some commit>] Not Found: [404] Not Found`

```bash
GIT_LFS_SKIP_SMUDGE=1 git reset --hard HEAD
git clean -fd
git lfs pull # You might need to download git-lfs first
```
