# MD_API Design

## Main.rs

> Registers handlers and business logic

1. When logging in, `load_user` information

2. User finds a new story, `add_title` modifies user and begins downloading
   
   1. `remove_title` counterpart

3. Begin reading. Client requests `download_chapter` for a story

4. `<img src="myAPI">` handle image reqs

## Web.rs

> Scrapes data and images

- scout title page (the essentials)

- download chapter

## User.rs

> Manages User JSON profiling

- load user, save user

- add title, remove title

## Library.rs

> Manages Local Library: JSON and Files

- load library, save library

- add title, remove title

- get title by (id, url)

- get new ID

### Storage.rs

> Used by Library.rs, handles downloading and managing files

- setup title (.json + cover.png) - delete title

- download chapter - delete chapter
