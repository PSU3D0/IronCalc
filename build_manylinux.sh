#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e
# Treat unset variables as an error when substituting.
set -u
# Pipe commands return the exit status of the last command in the pipe.
set -o pipefail

# --- Configuration ---
PYTHON_VERSION="3.12"
MANYLINUX_TAG="manylinux2014" # As per user output
ARCH="x86_64"
PROJECT_DIR=$(pwd)
MATURIN_SUBDIR="bindings/python"
OUTPUT_DIR="${PROJECT_DIR}/dist"
RUST_TOOLCHAIN="stable"

# Derived variables
PYTHON_INTERPRETER_PATH="/opt/python/cp${PYTHON_VERSION//./}-cp${PYTHON_VERSION//./}/bin/python"
DOCKER_IMAGE="quay.io/pypa/${MANYLINUX_TAG}_${ARCH}"
CONTAINER_MATURIN_PATH="/io/${MATURIN_SUBDIR}"
LEVELS=$(echo "${MATURIN_SUBDIR}" | awk -F/ '{print NF}')
RELATIVE_OUT_PATH=""
for (( i=0; i<$LEVELS; i++ )); do
  RELATIVE_OUT_PATH="../${RELATIVE_OUT_PATH}"
done
RELATIVE_OUT_PATH="${RELATIVE_OUT_PATH}dist"

# --- Sanity Checks ---
# ... (keep sanity checks as before) ...
if [ ! -f "${PROJECT_DIR}/${MATURIN_SUBDIR}/pyproject.toml" ]; then echo "ERROR: '${MATURIN_SUBDIR}/pyproject.toml' not found relative to ${PROJECT_DIR}. Run from monorepo root."; exit 1; fi
if [ ! -d "${PROJECT_DIR}/${MATURIN_SUBDIR}" ]; then echo "ERROR: Maturin subdir '${MATURIN_SUBDIR}' not found relative to ${PROJECT_DIR}."; exit 1; fi
command -v docker >/dev/null 2>&1 || { echo >&2 "ERROR: Docker required."; exit 1; }
if ! docker info > /dev/null 2>&1; then echo "ERROR: Docker daemon not running."; exit 1; fi

echo "--- Configuration ---"
# ... (keep echo statements as before) ...
echo "Monorepo Root:      ${PROJECT_DIR}"
echo "Maturin Subdir:     ${MATURIN_SUBDIR}"
echo "Output Directory:   ${OUTPUT_DIR}"
echo "Python Version:     ${PYTHON_VERSION}"
echo "Manylinux Tag:      ${MANYLINUX_TAG}"
echo "Architecture:       ${ARCH}"
echo "Docker Image:       ${DOCKER_IMAGE}"
echo "Python Path (cont): ${PYTHON_INTERPRETER_PATH}"
echo "Maturin Path (cont):${CONTAINER_MATURIN_PATH}"
echo "Relative Out Path:  ${RELATIVE_OUT_PATH}"
echo "Rust Toolchain:     ${RUST_TOOLCHAIN}"
echo "---------------------"
echo "--- Variables Dump ---"
echo "DOCKER_IMAGE=${DOCKER_IMAGE}"
echo "PROJECT_DIR=${PROJECT_DIR}"
echo "PLAT=${MANYLINUX_TAG}_${ARCH}"
echo "CONTAINER_MATURIN_PATH=${CONTAINER_MATURIN_PATH}"
echo "RELATIVE_OUT_PATH=${RELATIVE_OUT_PATH}"
echo "----------------------"

# --- Prepare ---
# ... (keep prepare steps as before) ...
echo ">>> Pulling Docker image: ${DOCKER_IMAGE} (if necessary)..."
docker pull "${DOCKER_IMAGE}"
echo ">>> Creating output directory: ${OUTPUT_DIR}"
mkdir -p "${OUTPUT_DIR}"
echo ">>> Cleaning previous builds (if any)..."
rm -f "${OUTPUT_DIR}"/*.whl

# --- Build ---
echo ">>> Starting build inside Docker container..."

# Removed the trailing '\' from each line inside the bash -c "..." command
docker run --rm \
    -v "${PROJECT_DIR}":/io \
    -w /io \
    -e PLAT="${MANYLINUX_TAG}_${ARCH}" \
    "${DOCKER_IMAGE}" \
    bash -c "
        echo '--- Inside Container ---' &&
        echo 'Current directory: \$(pwd)' &&
        echo 'Updating package list and installing dependencies...' &&
        yum update -y && yum install -y curl openssl-devel git &&
        echo 'Installing Rust using rustup...' &&
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain ${RUST_TOOLCHAIN} -y &&
        echo 'Sourcing Rust environment...' &&
        source \$HOME/.cargo/env &&
        echo 'Verifying cargo presence...' &&
        cargo --version &&
        echo 'Using Python:' &&
        ${PYTHON_INTERPRETER_PATH} --version &&
        echo 'Upgrading pip...' &&
        ${PYTHON_INTERPRETER_PATH} -m pip install --upgrade pip &&
        echo 'Installing Maturin...' &&
        ${PYTHON_INTERPRETER_PATH} -m pip install maturin &&
        echo 'Changing directory to Maturin project: ${CONTAINER_MATURIN_PATH}' &&
        cd '${CONTAINER_MATURIN_PATH}' && # Switched to single quotes just in case path had odd chars
        echo 'Current directory: \$(pwd)' &&
        echo 'Building with Maturin (output to ${RELATIVE_OUT_PATH})...' &&
        ${PYTHON_INTERPRETER_PATH} -m maturin build --release --out '${RELATIVE_OUT_PATH}' && # Switched to single quotes
        echo 'Build finished inside container.' &&
        echo '------------------------'
    "

# --- Post Build ---
# ... (keep post build steps as before) ...
echo ">>> Build process finished."
echo ">>> Listing built wheels in ${OUTPUT_DIR}:"
ls -lh "${OUTPUT_DIR}"
if [ \"\$(ls -A ${OUTPUT_DIR})\" ]; then echo '>>> Build successful!'; else echo '>>> Build failed or no wheels produced.'; exit 1; fi
exit 0