very simple blog generator!

prema 는 pretty markdown의 줄임말입니다.

tailwindcss와 flowbite를 기반으로 하고 있습니다.

### 명령어
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

### MD Directory 구조
``` plain
- {filename}.md (required)
- option.toml (optional)
  - nav
  - theme
  - footer
- [image files]
- [other md directories]
```
구조는 위와 같이 계층형으로 되어 있으며 하위 디렉토리에 option이 없을 경우 상위 디렉토리를 참조하게 됩니다.

theme의 경우 설정되어있지 않으면 상위theme를 따라가며 상위도 없다면 기본적으로 dark
md 파일은 한 directory에 반드시 1개만 있어야 하며 파일명은 상관 없음
image파일은 jpg, jpeg, png만 지원

### 페이지를 링크하는 법
``` markdown
[Link](rust "")
```

### 원리
디렉토리 하나를 하나의 페이지로 보고 있습니다.
디렉토리 하나에 하나의 md파일이 반드시 존재해야 하며 이미지가 있을 경우 이미지 파일들이 해당 md파일과 같이 있으면 좋습니다.

### md 파일
디렉토리에 한개만 있으면 됩니다.
없거나 2개 이상일경우 없는 디렉토리로 간주합니다.

### 태그하는 법
지원 예정
``` markdown
[#TAG]: # "android, rust"
```

### 작성시간 넣기
지원 예정
``` markdown
[#TIME]: # "24.06.23 10:23"
```

### 마크다운 문서파일들 변경하기
마크다운 파일들이 있는 루트 폴더를 지정하고 html이 들어갈 루트 폴더를 지정하면 된다.
``` shell
prema html {root_path} {html_path}
```

### 메인 아이콘(홈으로 갈 수 있는..)
아직 미지원

### 내비
option.toml에 아래와 같이 적으면 되고 2depth까지만 지원합니다.
``` toml
[nav]
home = "/"
service.etc1 = "etc1"
service.etc2 = "etc2"
service.etc3 = "etc3"
menu = "menu"
end = "end"
```

### 테마
현재 다크모드를 설정할 수 있습니다.
``` toml
[theme]
night = true
```

### 푸터
하단에 들어갈 문장을 작성할 수 있습니다.
sns는 현재 5개 지원합니다.
``` toml
[footer]
title = "© 2025 Prema. All Rights Reserved"
sns.facebook = ""
sns.discord = ""
sns.twitter = ""
sns.github = ""
sns.dribble = ""
```


```html
<div class="flex-none hidden w-64 pl-8 mr-8 xl:text-sm xl:block">

          <div class="flex overflow-y-auto sticky top-28 flex-col justify-between pt-10 pb-6 h-[calc(100vh-5rem)]">
            <div class="mb-8">
              <h4 class="pl-2.5 mb-2 text-sm font-semibold tracking-wide text-gray-900 uppercase dark:text-white lg:text-xs">On this page</h4>
              <nav id="TableOfContents">
  <ul>
    <li><a href="#dropdown-example">Dropdown example</a></li>
    <li><a href="#dropdown-hover">Dropdown hover</a>
      <ul>
        <li><a href="#delay-duration">Delay duration</a></li>
      </ul>
    </li>
    <li><a href="#dropdown-divider">Dropdown divider</a></li>
    <li><a href="#dropdown-header">Dropdown header</a></li>
    <li><a href="#multi-level-dropdown">Multi-level dropdown</a></li>
    <li><a href="#dropdown-with-checkbox">Dropdown with checkbox</a>
      <ul>
        <li><a href="#background-hover">Background hover</a></li>
        <li><a href="#helper-text">Helper text</a></li>
      </ul>
    </li>
    <li><a href="#dropdown-with-radio">Dropdown with radio</a>
      <ul>
        <li><a href="#background-hover-1">Background hover</a></li>
        <li><a href="#helper-text-1">Helper text</a></li>
      </ul>
    </li>
    <li><a href="#dropdown-with-toggle-switch">Dropdown with toggle switch</a></li>
    <li><a href="#dropdown-with-scrolling">Dropdown with scrolling</a></li>
    <li><a href="#dropdown-with-search">Dropdown with search</a></li>
    <li><a href="#menu-icon">Menu icon</a></li>
    <li><a href="#notification-bell">Notification bell</a></li>
    <li><a href="#user-avatar">User avatar</a>
      <ul>
        <li><a href="#avatar-with-name">Avatar with name</a></li>
      </ul>
    </li>
    <li><a href="#dropdown-navbar">Dropdown navbar</a></li>
    <li><a href="#dropdown-datepicker">Dropdown datepicker</a></li>
    <li><a href="#sizes">Sizes</a></li>
    <li><a href="#placement">Placement</a>
      <ul>
        <li><a href="#double-placement">Double placement</a></li>
      </ul>
    </li>
    <li><a href="#dropdown-offset">Dropdown offset</a>
      <ul>
        <li><a href="#distance">Distance</a></li>
        <li><a href="#skidding">Skidding</a></li>
      </ul>
    </li>
    <li><a href="#more-examples">More examples</a></li>
    <li><a href="#javascript-behaviour">JavaScript behaviour</a>
      <ul>
        <li><a href="#object-parameters">Object parameters</a></li>
        <li><a href="#options">Options</a></li>
        <li><a href="#methods">Methods</a></li>
        <li><a href="#example">Example</a></li>
        <li><a href="#html-markup">HTML Markup</a></li>
        <li><a href="#typescript">TypeScript</a></li>
      </ul>
    </li>
  </ul>
</nav>

              <aside class="w-52 mt-6 mx-auto border-t border-gray-200 dark:border-gray-700 pt-8">
                <a href="https://www.enhanceui.com/?ref=flowbite-sidebar" class="block rounded-lg" rel="nofollow noopener noreferrer" target="_blank"> 
                  <img src="/docs/images/book-light.svg" class="shadow-sm mb-3 w-36 dark:hidden" alt="Enhance UI book cover light mode">
                  <img src="/docs/images/book-dark.svg" class="shadow-sm mb-3 w-36 hidden dark:block" alt="Enhance UI book cover dark mode">
                  <h4 class="text-base font-semibold text-gray-900 mb-1.5 dark:text-white">Learn Design Concepts</h4>
                  <p class="text-gray-500 dark:text-gray-400">Make better Flowbite pages by learning the fundamentals of design</p>
                  <div class="border-t border-gray-200 dark:border-gray-700 pt-2 mt-2">
                    <h5 class="font-medium text-gray-900 dark:text-white">Teach Me Design</h5>
                    <p class="text-gray-500 dark:text-gray-400">by Adrian Twarog</p>
                  </div>
                </a>
              </aside>
            </div>
          </div>

        </div>

        ```
        