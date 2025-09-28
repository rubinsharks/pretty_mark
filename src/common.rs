use std::{fs, path::Path};

/// ``` 코드 에 해당하는 부분들이 제대로 나타나지 않기 때문에
/// 아래 코드로 완성된 코드를 재배치한다.
pub fn remove_code_indentation(html: String) -> String {
    let code_tag_re = regex::Regex::new(r#"(<code[^>]*>)([\s\S]*?)</code>"#).unwrap();

    let remove_first = code_tag_re
        .replace_all(html.as_str(), |caps: &regex::Captures| {
            let code_open_tag = &caps[1]; // <code class="...">
            let code_content = &caps[2];  // 내용 부분

            let lines: Vec<&str> = code_content.split('\n').collect();

            let min_indent = lines
                .iter()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.chars().take_while(|c| *c == ' ' || *c == '\t').count())
                .min()
                .unwrap_or(0);

            let new_content: String = lines
                .into_iter()
                .filter(|line| !line.trim().is_empty())
                .map(|line| {
                    if line.len() >= min_indent {
                        &line[min_indent..]
                    } else {
                        line
                    }
                })
                .collect::<Vec<&str>>()
                .join("\n");

            format!("{}{}{}", code_open_tag, new_content, "</code>")
        })
        .to_string();

    remove_first
}

pub fn copy_img_files_to_path(src_dir: &Path, dest_dir: &Path) -> Result<(), String> {
  // src_dir이 디렉토리인지 확인
  if !src_dir.is_dir() {
      return Err(format!("Source directory does not exist or is not a directory: {:?}", src_dir));
  }

  // dest_dir이 없으면 생성
  if !dest_dir.exists() {
      fs::create_dir_all(dest_dir)
          .map_err(|e| format!("Failed to create destination directory: {}", e))?;
  }

  // src_dir 읽기
  for entry_res in fs::read_dir(src_dir)
      .map_err(|e| format!("Failed to read source directory: {}", e))? {

      let entry = entry_res.map_err(|e| format!("Failed to read directory entry: {}", e))?;
      let path = entry.path();

      // 파일이고 확장자가 jpg, jpeg, png인지 확인 (대소문자 구분 없이)
      if path.is_file() {
          if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
              let ext_lower = ext.to_lowercase();
              if ext_lower == "jpg" || ext_lower == "jpeg" || ext_lower == "png" || ext_lower == "svg" {
                  let file_name = path.file_name()
                      .ok_or_else(|| "Failed to get file name".to_string())?;
                  let dest_path = dest_dir.join(file_name);

                  fs::copy(&path, &dest_path)
                      .map_err(|e| format!("Failed to copy {:?} to {:?}: {}", path, dest_path, e))?;
              }
          }
      }
  }

  Ok(())
}

pub trait SlashNormalize {
    fn ensure_slashes(&self) -> String;
}

impl SlashNormalize for str {
    fn ensure_slashes(&self) -> String {
        let mut s = self.to_string();
        if !s.starts_with('/') {
            s.insert(0, '/');
        }
        if !s.ends_with('/') {
            s.push('/');
        }
        s
    }
}

impl SlashNormalize for String {
    fn ensure_slashes(&self) -> String {
        self.as_str().ensure_slashes()
    }
}