#!/bin/sh

function help {
    echo "usage: $(basename $0) -o OUTPUT_FILE -l LOADER_FILE -i INITRD_FILE [-s IMAGE_SIZE] [-a TARGET_ARCH]"
    exit 0
}

function error {
    echo "error: $@"
    exit 1
}

function cleanup {
    # unmount loopdev if it was mounted
    [[ "$MOUNT_SUCCESS" = "true" ]] && "$PRIV_EXE" umount "$LOOP_DEV"

    # detach loopdev if it was allocated
    [[ "$LOSETUP_SUCCESS" = "true" ]] && "$PRIV_EXE" losetup -d "$LOOP_DEV"

    # delete temp directory if it was created
    [[ "$TEMP_DIR_SUCCESS" = "true" ]] && rm -rf "$TEMP_DIR"
}

trap cleanup EXIT


# Select doas or sudo to use for privilege escaltion
if command -v doas &> /dev/null
then
    PRIV_EXE="doas"    
elif command -v sudo &> /dev/null
then
    PRIV_EXE="sudo"
else
    error "either sudo or doas need to be available"
fi


while getopts ":o:l:i:s:a:h" ARG; do
    case "$ARG" in 
        o) OUTPUT_FILE="$OPTARG" ;;
        l) LOADER_FILE="$OPTARG" ;;
        i) INITRD_FILE="$OPTARG" ;;
        a) TARGET_ARCH="$OPTARG" ;;
        s) IMAGE_SIZE="$OPTARG" ;;
        h) help ;;
    esac
done

shift $(expr $OPTIND - 1 )

[[ -z "${OUTPUT_FILE+x}" ]] && error "output file must be provided"
[[ -z "${LOADER_FILE+x}" ]] && error "loader file must be provided"
[[ -z "${INITRD_FILE+x}" ]] && error "initrd file must be provided"

[[ -z "${TARGET_ARCH+x}" ]] && TARGET_ARCH="x86_64"
[[ -z "${IMAGE_SIZE+x}" ]] && IMAGE_SIZE="64M"

if [ "$TARGET_ARCH" = "x86_64" ]; then
    EFI_LOADER_NAME="bootx64.efi"
elif [ "$TARGET_ARCH" = "i686" ]; then 
    EFI_LOADER_NAME="bootia32.efi"
else 
    error "unknown target architecture: $TARGET_ARCH"
fi

# create a temp directory
TEMP_DIR=$(mktemp -d)
[[ ! "$TEMP_DIR" || ! -d "$TEMP_DIR" ]] && error "failed to create temp directory"

TEMP_DIR_SUCCESS=true

# allocate a loop device
LOOP_DEV=$(losetup -f || echo "error")
[[ "$LOOP_DEV" = "error" ]] && error "failed to obtain loop device"

# create an empty disk image
dd if=/dev/zero of="$OUTPUT_FILE" bs="$IMAGE_SIZE" count=1 || error "dd failed"

# use fdisk to crate the GPT and an EFI System partition
printf "g\nn\n1\n2048\n\nt\nEFI System\nw\n" | fdisk "$OUTPUT_FILE" &> /dev/null || error "fdisk failed"

"$PRIV_EXE" losetup -o1048576 "$LOOP_DEV" "$OUTPUT_FILE" || error "losetup failed"

LOSETUP_SUCCESS=true

"$PRIV_EXE" mkfs.vfat -F32 "$LOOP_DEV" || error "mkfs.vfat failed"

"$PRIV_EXE" mount "$LOOP_DEV" "$TEMP_DIR" || error "mount failed"

MOUNT_SUCCESS=true

"$PRIV_EXE" mkdir -p "$TEMP_DIR/efi/boot/"
"$PRIV_EXE" mkdir -p "$TEMP_DIR/yeetos/"

"$PRIV_EXE" cp "$LOADER_FILE" "$TEMP_DIR/efi/boot/$EFI_LOADER_NAME"
"$PRIV_EXE" cp "$INITRD_FILE" "$TEMP_DIR/yeetos/initrd"


