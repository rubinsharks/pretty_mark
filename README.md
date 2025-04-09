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
  - basic
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

### 마크다운 문서파일들 변경하기
마크다운 파일들이 있는 루트 폴더를 지정하고 html이 들어갈 루트 폴더를 지정하면 된다.
``` shell
prema html {root_path} {html_path}
```

### 메인 아이콘(홈으로 갈 수 있는..)
아직 미지원

### 베이직
created는 아래와 같은 포맷으로 작성하면 html에 자동으로 작성일이 추가됩니다.
tag의 경우 나중에 tag list를 통해 tag별로 게시물을 확인할 수 있습니다.
``` toml
created = "yyyy-MM-dd hh:mm:ss"
tag = "food"
```

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
