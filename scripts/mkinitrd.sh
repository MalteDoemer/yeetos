#!/bin/sh

function help {
    echo "usage $(basename $0) -o OUTPUT_FILE -k KERNEL_FILE "
    exit 0
}

function error {
    echo error: $@
    exit 1
}

function cleanup {
    rm -rf "$TEMP_DIR"
}

trap cleanup EXIT

while getopts ":o:k:h" ARG; do
    case "$ARG" in 
        o) OUTPUT_FILE="$OPTARG" ;;
        k) KERNEL_FILE="$OPTARG" ;;
        h) help ;;
    esac
done

shift $(expr $OPTIND - 1 )

[[ -z "${KERNEL_FILE+x}" ]] && error "kernel file must be provided"

[[ -z "${OUTPUT_FILE+x}" ]] && error "output file must be provided" 

TEMP_DIR=$(mktemp -d)

[[ ! "$TEMP_DIR" || ! -d "$TEMP_DIR" ]] && error "failed to create temp directory" 

cp "$KERNEL_FILE" "$TEMP_DIR/kernel"

cd "$TEMP_DIR" && tar -cf "$OUTPUT_FILE" $(ls)
