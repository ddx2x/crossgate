# crossgate

## build

```
docker buildx build \
  --platform linux/arm/v7,linux/arm64/v8,linux/386,linux/amd64,linux/ppc64le \
  -t stream:0.1\
  -f Dockerfile.stream \
  .
```
