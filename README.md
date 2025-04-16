Very simple static site generator from markdown!

Prema is short for Pretty Markdown.

It is based on Tailwind CSS and Flowbite.

### Install
You can install this package using Cargo:
``` bash
cargo install prema
```

### Command
``` bash
# generate html
# make hierarchy htmls based on selected directory
# make md files to html files in html_directory
prema html {md_directory} {html_directory}

# generate set of md
# make {name}.md, option.toml
prema new {name}

# setting tags
prema new {name} --tags "ios, android"
```

### MD Directory Structure
``` plain
- {filename}.md (required)
- option.toml (optional)
  - basic
  - nav
  - theme
  - footer
- [image files]
- [other md directories]
```
The structure is hierarchical as shown above. If an option is not present in a subdirectory, it will inherit from its parent directory.

For the theme setting, if it is not explicitly defined, it will follow the theme of the parent directory. If no theme is set in any parent directories, the default is dark.

Each directory must contain exactly one Markdown (.md) file, and the filename does not matter.

Only image files in JPG, JPEG, or PNG formats are supported.

### Principle
Each directory is treated as a single page.
There must be exactly one Markdown (.md) file in each directory.
If there are image files, it's best to place them in the same directory as the corresponding Markdown file.

### Link
- Page Link
  
If you want to link to the rust folder, you can do it like this,
``` markdown
[Link](rust "")
```
If there's a rust folder inside the language directory, you can link to it like this,
``` markdown
[Link](language/rust "")
```
You can link to it like above.
- Image Link

After adding the image file,
``` markdown
![Alt text](image.jpeg "Optional title")
```
As shown above, simply add an exclamation mark (!), followed by the alt text, and then the file name, such as image.jpeg.

### .md File
There should be exactly one Markdown file in each directory.
If there are none or more than one, the directory will be considered as one without an .md file.

### Converting Markdown document files
You need to specify the root folder containing the Markdown files and the root folder where the HTML files will be placed.
``` shell
prema html {root_path} {html_path}
```

### Main Icon(This can go home..)
Not supported yet..

### Basic
- title
  - Represents the name of the site or blog.
  - If no title is provided, the first line with a heading (#, ##, ...) will be recognized as the title.
  - The title is used when the list corresponding to the tags is displayed.
- created
  - If you write in the "yyyy-MM-dd hh:mm:ss" format, the creation date will be automatically added to the HTML.
- tag
  - You will be able to view posts by tag through the tag list later.

``` toml
[basic]
title = "Prema"
created = "yyyy-MM-dd hh:mm:ss"
tag = "food"
```

### Nav
You can define options in option.toml as shown below.
Only up to 2 levels of depth are supported.
This is used to generate a menu bar for the site.
``` toml
[nav]
# if you want to link to home page,
home = "/"

# if you want to 2depth menu,
service.etc1 = "etc1"
service.etc2 = "etc2"
service.etc3 = "etc3"

menu = "menu"
end = "end"

# if you want to name with space,
"about me" = "about_me"
my."about me" = "my/about_me"
"my profile"."about me" = "my_profile/about_me"
```

### Theme
You can configure whether dark mode is enabled as shown below.
``` toml
[theme]
night = true
```

### Footer
You can write a custom message to be displayed at the bottom of the page.
Currently, 5 social media platforms are supported.
``` toml
[footer]
title = "© 2025 Prema. All Rights Reserved"
sns.facebook = ""
sns.discord = ""
sns.twitter = ""
sns.github = ""
sns.dribble = ""
```
