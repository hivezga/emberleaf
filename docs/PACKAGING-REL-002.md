# REL-002: Packaging Bootstrap

## Deliverables

- AppImage + .deb artifacts
- SHA256 checksums
- CI upload on tags `v*`
- INSTALL-Linux.md with troubleshooting

## Local Build (examples)

```bash
# AppImage
scripts/release/package_appimage.sh

# .deb
scripts/release/package_deb.sh
```

## Release Flow

1. Merge to `main`
2. Tag: `git tag v1.0.0 && git push origin v1.0.0`
3. CI builds & uploads artifacts and SHASUMS256.txt
4. Publish release notes
