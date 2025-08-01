partition_manager::macros::create_partition_map!(
    name: InternalStorageConfig,
    map_name: InternalStorageMap,
    variant: "bootloader",
    manifest: "int_flash.toml"
);

partition_manager::macros::create_partition_map!(
    name: ExternalStorageConfig,
    map_name: ExternalStorageMap,
    variant: "bootloader",
    manifest: "ext_flash.toml"
);
