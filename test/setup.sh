#!/usr/bin/env bash
set -Eeuo pipefail

clean_env() {
    local keep_vars=(
        MASTR_EXPORT_TAG
        PATH
        PWD
        TESTDIR
    )
    local keep_vars_expr
    keep_vars_expr=$(IFS='|'; echo "${keep_vars[*]}")

    unset $(env | grep -v -E "^(${keep_vars_expr})=[^=]*$" | cut -d= -f1)
}

remove_err_files() {
    find . -name '*.err' -exec rm '{}' \;
}

build_image() {
    local output
    output=$(podman build --quiet --tag "${MASTR_EXPORT_TAG}" "${TESTDIR}/..")
    if [[ ! "${output}" =~ ^[0-9a-f]{64}$ ]]; then
        echo "Failed to build image"
        exit 1
    fi
}

mastr-export() {
    mkdir -p work
    podman run --quiet --rm --transient-store \
        --userns=keep-id:uid=65535,gid=65535 \
        --volume ./work/:/mnt/work/:rw,Z,U \
        "${MASTR_EXPORT_TAG}" "$@"
}

if [[ -z "${SKIP_BUILD:-}" ]]; then
    build_image
fi

clean_env
remove_err_files
