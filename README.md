Very simple static site generator from markdown!

Prema is short for Pretty Markdown.

It is based on Tailwind CSS and Flowbite.

### Install

You can install this package using Cargo:

```bash
cargo install prema
```

### Command

```bash
# generate html
# make hierarchy htmls based on selected directory
# make md files to html files in html_directory
prema html {target_directory} {html_directory}

# generate set of md
# make directory of {name} contains {name}.md, option.toml
prema new {name}

```

### Structure

When the conversion is performed, it searches subfolders based on the target directory.
If an index.toml or [index.md](http://index.md/) file exists inside a folder, an index.html file is generated to create a page.
If both files exist, index.toml takes precedence.

# index.toml

### root

In index.toml, the first layout must always be root.
Therefore, a root layout must exist, and the structure should be designed so that layouts propagate outward from the root layout.

```toml
[root]
shape = "column"
width = "100%"
height = "100%"
dark = false

```

### layout

Layouts have a required shape property and optional properties that can be set as needed: width, height, background, path, value, and dark.

- width, height: Adjust the size; can be specified in px, %, or wrap.
- background: Sets the background color.
- path: Makes the layout a clickable button that navigates to the specified value when pressed.
- value: If specified, allows setting values for the layout and its sublayouts using {}.
- dark: Determines whether the layout uses a dark theme. If not set, it inherits the dark theme from the parent layout.

```toml
[root.contents]
shape = "column"
width = "200px"
height = "100%"
background = "#000"
path = "#"
value = { skill_title = "1", skill_image = "duck.jpeg", skill_summary = "index.md" }
dark = false

```

### sub layout

To place a layout as a child of another layout, extend it from the parent layout key.
If a sublayout is needed under the root, extend it as root.{} by inserting the desired name inside the {}.
Multiple sublayouts can exist.

A sublayout is affected by the structure of its parent layout.
If the parent layout is a column, it will be arranged vertically, and if it is a row, it will be arranged horizontally.

```toml
[root]
shape = "column"
width = "100%"
height = "100%"
dark = false

[root.child1]
shape = "text"
width = "100%"
height = "wrap"
text = "Title"
color = "#fff"
size = "24px"
weight = "bold"
left_outer_padding = "20px"

[root.child2]
shape = "markdown"
width = "100%"
height = "100%"
markdown_path = "title.md"
horizontal_outer_padding = "20px"

```

### nav

In nav, title refers to the text displayed on the left, and headers refers to the menus displayed on the right.
Menu labels cannot duplicate reserved keywords in the layout (such as width, height, dark, etc.).
If you want to add a submenu to a menu, you can define it like service.etc1 = "" as shown in the example below.
If a submenu exists, defining a non-submenu item for the same menu (e.g., service = "") may cause conflicts.

```toml
[root.nav]
shape = "nav"
width = "100%"
height = "70px"
title = "STAR"
headers = ["home", "service", "menu", "end", "about me"]
home = "/"
end = "doc"
service.etc1 = "etc1"
service.etc2 = "etc2"
service.etc3 = "etc3"
"about me".my = "about_me/my"

```

### embed

Specifies a parent or current layout that is not included in the root.

This is necessary when the same layout is used across multiple views, typically for setting headers or footers.

### column, row, box

Sublayouts can be included, and the arrangement direction is determined by the shape.

```toml
[root]
shape = "column"
width = "100%"
height = "100%"
dark = false

```

### list_column, list_row

In the case of a list, you can specify a layout.
The layout specified can receive values through values.
The injected values can also be passed to the specified layout’s sublayouts using {}.
A layout is generated for each value in values, and whether they are arranged vertically or horizontally is determined by whether it is list_column or list_row.

```toml
[root.contents.skills]
shape = "list_row"
width = "100%"
height = "200px"
layout = "skill_layout"
order_by = "skill_title" # title, date, author, ...
values = [
  { skill_title = "1", skill_image = "duck.jpeg", skill_summary = "index.md" },
  { skill_title = "2", skill_image = "2.png", skill_summary = "skill.md" },
  { skill_title = "3", skill_image = "3.png", skill_summary = "1.md" },
  { skill_title = "3", skill_image = "3.png", skill_summary = "1.md" },
  { skill_title = "3", skill_image = "3.png", skill_summary = "1.md" },
  { skill_title = "3", skill_image = "3.png", skill_summary = "1.md" },
  { skill_title = "3", skill_image = "3.png", skill_summary = "1.md" },
  { skill_title = "3", skill_image = "3.png", skill_summary = "1.md" },
]
background = "#0ff"

[skill_layout]
shape = "column"
width = "100px"
height = "250px"
background = "#00f"

[skill_layout.skill_title]
shape = "text"
width = "100%"
height = "wrap"
size = "24px"
text = "{skill_title}"
color = "#fff"
family = "montserrat"

[skill_layout.skill_summary]
shape = "markdown"
width = "100%"
height = "wrap"
markdown_path = "{skill_summary}"

[skill_layout.skill_image]
shape = "image"
width = "200px"
height = "200px"
image_path = "{skill_image}"
content_size = "cover" # cover, contain, fill

```

### mdlist_column, mdlist_row

Creates a list of Markdown files located in the specified directory.

When the user taps an item in the list, it navigates to the corresponding Markdown file.

The frontmatter and filename of each Markdown file can be accessed and used in the form of {}.

```toml
[root]
shape = "column"
width = "100%"
height = "100%"
dark = true

[root.nav]
shape = "embed"
layout = "nav"

[root.contents]
shape = "mdlist_column"
width = "100%"
height = "wrap"
layout = "markdown_row"
files = "*.md"
horizontal_padding = "20px"

[root.footer]
shape = "embed"
layout = "footer"

[markdown_row]
shape = "row"
width = "wrap"
height = "wrap"

[markdown_row.title]
shape = "text"
width = "wrap"
height = "wrap"
size = "16px"
text = "{title}"
path = "{filename}"
color = "#fff"
```

### text

Text properties such as content, size, font, and alignment can be set. Sublayouts cannot be included.

```toml
[root.contents.title]
shape = "text"
width = "wrap"
height = "wrap"
size = "24px"
text = "Jumbotron!"
color = "#000"
family = "montserrat"
weight = "bold"
path = "#"
vertical_align = "center" # top, center, bottom
horizontal_align = "center" # left, center, right

```

### image

Displays an image and allows setting image_path and content_size. Sublayouts cannot be included.

```toml
[skill_layout.skill_image]
shape = "image"
width = "200px"
height = "200px"
image_path = "image.jpeg" # jpeg, jpg, png, svg
content_size = "cover" # cover, contain, fill

```

### markdown

A Markdown file can be applied, and markdown_path can be set. Sublayouts cannot be included.

```toml
[skill_layout.skill_summary]
shape = "markdown"
width = "100%"
height = "wrap"
markdown_path = "info.md"

```

### grid

- Not yet supported

# [index.md](http://index.md/)

If a folder does not have an index.toml configured, [index.md](http://index.md/) is converted instead, and its content is treated as Markdown.

# Markdown

### Link

- Page Link

If you want to link to the rust folder, you can do it like this,

```markdown
[Link](rust "")

```

If there's a rust folder inside the language directory, you can link to it like this,

```markdown
[Link](language/rust "")

```

You can link to it like above.

- Image Link

After adding the image file,

```markdown
![Alt text](image.jpeg "Optional title")

```

As shown above, simply add an exclamation mark (!), followed by the alt text, and then the file name, such as image.jpeg.

### Supported Markdown

```
Heading
# Heading Text
## Heading Text
### Heading Text
#### Heading Text
##### Heading Text
###### Heading Text

List
- ItemText
* ItemText

Line
***
---

Link
[Text](MdLink "Title")
[Text](HttpLink "Title")
![Alt](ImageLink "Title")

TextStyle
**Strong**
*Emphasis*

BlockQuote
> Text

Code
⠀``` rust
fn main() {

}
⠀```

Table
| Month    | Savings |
| -------- | ------  |
| January  | $250    |
| February | $80     |
| March    | $420    |

```