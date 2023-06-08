# MD_API Design

## Main.rs

- main --> registers handlers

## User.rs

> Manages User JSON profiling

- load user, save user

- add title, remove title

## Library.rs

> Manages Library JSON

- load library, save library

- add title, remove title

- get title by (id, url)

- get new ID

## Storage.rs

> Used by Library.rs, handles downloading and managing files

- setup title (.json + cover.png) - delete title

- download chapter - delete chapter

## Title, Chapter

> Title and Chapter Structs

- constructors
