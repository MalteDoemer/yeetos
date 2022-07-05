#!/bin/sh

function help {
    echo "usage $(basename $0) [-o OUTPUT_FILE] FILES..."
}

while getopts ":o:h" ARG; do
    case "$ARG" in 
        o) OUTPUT_FILE="$OPTARG" ;;
        h) help ;;
    esac
done

if [[ -z "${OUTPUT_FILE+x}" ]]; then 
    OUTPUT_FILE="$(pwd)/initrd"
fi

shift $(expr $OPTIND - 1 )


TEMP_DIR=$(mktemp -d)

if [[ ! "$TEMP_DIR" || ! -d "$TEMP_DIR" ]]; then
    echo "failed to create temp directory" 
    exit 1 
fi

function cleanup {
    rm -rf "$TEMP_DIR"
}

trap cleanup EXIT

cp $@ "$TEMP_DIR"

cd "$TEMP_DIR" && tar -cf "$OUTPUT_FILE" ./*