#!/bin/sh

function help {
    echo "usage: $(basename $0) -o OUTPUT_FILE -l LOADER_FILE -i INITRD_FILE"
    exit 0
}

function error {
    echo "error: $@"
    exit 1
}

function cleanup {
    rm -rf "$TEMP_DIR"
}

trap cleanup EXIT

while getopts ":o:l:i:h" ARG; do
    case "$ARG" in 
        o) OUTPUT_FILE="$OPTARG" ;;
        l) LOADER_FILE="$OPTARG" ;;
        i) INITRD_FILE="$OPTARG" ;;
        h) help ;;
    esac
done

shift $(expr $OPTIND - 1 )

[[ -z "${OUTPUT_FILE+x}" ]] && error "output file must be provided"
[[ -z "${LOADER_FILE+x}" ]] && error "loader file must be provided"
[[ -z "${INITRD_FILE+x}" ]] && error "initrd file must be provided"

TEMP_DIR=$(mktemp -d)

[[ ! "$TEMP_DIR" || ! -d "$TEMP_DIR" ]] && error "failed to create temp directory"

mkdir -p "$TEMP_DIR/boot/grub"
mkdir -p "$TEMP_DIR/yeetos"

echo "set timeout=0
menuentry \"yeetos\" {
    multiboot2 /yeetos/loader
    module2 /yeetos/initrd initrd
    boot
}" > "$TEMP_DIR/boot/grub/grub.cfg"

cp "$LOADER_FILE" "$TEMP_DIR/yeetos/loader"
cp "$INITRD_FILE" "$TEMP_DIR/yeetos/initrd"

grub-mkrescue -o "$OUTPUT_FILE" "$TEMP_DIR" -quiet &> /dev/null
