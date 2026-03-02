group "default" {
  targets = ["rust-ci", "node-ci", "python-ext-ci", "tauri-ci"]
}

variable "VERSION" {
  default = "latest"
}

target "rust-ci" {
  context = "."
  dockerfile = "examples/rust-lib/Dockerfile.devflow"
  platforms = ["linux/amd64", "linux/arm64"]
  tags = ["devflow-ci-rust:${VERSION}"]
  output = ["type=image,compression=zstd,compression-level=3"]
}

target "node-ci" {
  context = "."
  dockerfile = "examples/node-ts/Dockerfile.devflow"
  platforms = ["linux/amd64", "linux/arm64"]
  tags = ["devflow-ci-node:${VERSION}"]
  output = ["type=image,compression=zstd,compression-level=3"]
}

target "python-ext-ci" {
  context = "."
  dockerfile = "examples/python-ext/Dockerfile.devflow"
  platforms = ["linux/amd64", "linux/arm64"]
  tags = ["devflow-ci-python:${VERSION}"]
  output = ["type=image,compression=zstd,compression-level=3"]
}

target "tauri-ci" {
  context = "."
  dockerfile = "examples/tauri/Dockerfile.devflow"
  platforms = ["linux/amd64", "linux/arm64"]
  tags = ["devflow-ci-tauri:${VERSION}"]
  output = ["type=image,compression=zstd,compression-level=3"]
}
