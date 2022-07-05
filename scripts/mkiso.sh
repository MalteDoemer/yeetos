#!/bin/sh

function help {
    echo "usage $(basename $0) [-o OUTPUT_FILE] LOADER_FILE INITRD_FILE"
}

while getopts ":o:h" ARG; do
    case "$ARG" in 
        o) OUTPUT_FILE="$OPTARG" ;;
        h) help ;;
    esac
done

if [[ -z "${OUTPUT_FILE+x}" ]]; then 
    OUTPUT_FILE="$(pwd)/yeetos.iso"
fi

shift $(expr $OPTIND - 1 )

LOADER_FILE="$1"
INITRD_FILE="$2"

TEMP_DIR=$(mktemp -d)

if [[ ! "$TEMP_DIR" || ! -d "$TEMP_DIR" ]]; then
    echo "failed to create temp directory" 
    exit 1 
fi

function cleanup {
    rm -rf "$TEMP_DIR"
}

trap cleanup EXIT

mkdir -p "$TEMP_DIR/boot/grub"
mkdir -p "$TEMP_DIR/yeetos"

echo "set timeout=0
menuentry \"yeetos\" {
    multiboot2 /yeetos/loader
    module2 /yeetos/initrd
    boot
}" > "$TEMP_DIR/boot/grub/grub.cfg"

cp "$LOADER_FILE" "$TEMP_DIR/yeetos/loader"
cp "$INITRD_FILE" "$TEMP_DIR/yeetos/initrd"

grub-mkrescue -o "$OUTPUT_FILE" "$TEMP_DIR" -quiet &> /dev/null
