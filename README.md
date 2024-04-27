# diary-cli
---
> *A powerful cli for documenting and keeping a diary.*

## How to install
---
1. run
    ```sh
        cargo install --locked diary-cli
    ```
2. Try it out
    ```sh
        diary-cli --help`
    ```
## Anatomy of a Diary Entry
---
### Entry Metadata
---
> useful meta-data about that diary entry
```toml
uid = "<a unique identifier that is used for `MOC`s and also defines the file name the entry exports as"
date = 1000-01-01 # date that it occured

title = "<the title of the diary entry>"
description = "<description>"
tags = [
    "a",
    "bunch",
    "of",
    "tags",
    "to",
    "attach",
    "(used for both `Obsidian.md` and also for `MOC`s)"
]
notes = [
    "a bunch of summaries of the entry",
    "in case you're lazy",
]
```
### Sections
---
> a section is a paragraph or topic within a diary entry
```toml
[[section]]
title = "title of / the topic of this section"
contents = """
the string contents of this diary entry
yeah
"""
```

## Anatomy of a `MOC`
---
a `MOC` or a 'Map of Contents' is a markdown file that contains links to other mocs or entries

sort of like a directory or folder

### MOC Metadata
---
```toml
is-moc = true # tells `diary-cli` that this is a `MOC` file

[moc]
title = "MOCs of 2023"
uid = "2023-mocs"
description = "description about this MOC"
notes = ["notes about this MOC"]
tags = [
    "Tags for this MOC that allow it to be searchable or indexed by other MOCs",
]
```
### Collections
---
> like an entry section, but with markdown links instead
```toml
[[collection]]
title = "title / topic of the collection"
notes = ["notes about this collection section"]
include = [ # tags of entries or `MOC`s to include in the collection
    "interesting",
    "moc",
]
```
