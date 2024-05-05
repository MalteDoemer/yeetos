#!/bin/sh

function help {
    echo "usage $(basename $0) -o OUTPUT_FILE -k KERNEL_FILE -c CMDLINE_FILE -f FONT_FILE"
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

while getopts ":o:k:c:f:h" ARG; do
    case "$ARG" in 
        o) OUTPUT_FILE="$OPTARG" ;;
        k) KERNEL_FILE="$OPTARG" ;;
        c) CMDLINE_FILE="$OPTARG" ;;
        f) FONT_FILE="$OPTARG" ;;
        h) help ;;
    esac
done

shift $(expr $OPTIND - 1 )

[[ -z "${CMDLINE_FILE+x}" ]] && error "kernel command line file must be provided (-c CMDLINE_FILE)"

[[ -z "${KERNEL_FILE+x}" ]] && error "kernel file must be provided (-k KERNEL_FILE)"

[[ -z "${OUTPUT_FILE+x}" ]] && error "output file must be provided (-o OUTPUT_FILE)"

[[ -z "${FONT_FILE+x}" ]] && error "font file must be provided (-o FONT_FILE)"

TEMP_DIR=$(mktemp -d)

[[ ! "$TEMP_DIR" || ! -d "$TEMP_DIR" ]] && error "failed to create temp directory" 

cp "$KERNEL_FILE" "$TEMP_DIR/kernel"

cp "$CMDLINE_FILE" "$TEMP_DIR/cmdline"

cp "$FONT_FILE" "$TEMP_DIR/font.psf"

strip -dx "$TEMP_DIR/kernel"

cd "$TEMP_DIR" && tar -cf "$OUTPUT_FILE" $(ls)
