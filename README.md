# emoji-gen

A script written in Rust to generate a ZIP with right formating for a emoji pack.

### Usage

Put all emojis you want to import in a folder, use subfolders for different groups.

```a
cargo install emoji-gen
emoji-gen --folder . [optionally add --group "GroupName"]
```
upload the zip file to calckey and import it.