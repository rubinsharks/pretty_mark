
pub fn remove_frontmatter(input: &String) -> String {
    let lines = input.lines();

    // 첫 줄이 '-'만 반복된 문자열인지 확인 (공백 제거 후)
    if let Some(first) = lines.clone().next() {
        let first_trimmed = first.trim();
        if !first_trimmed.chars().all(|c| c == '-') {
            return input.clone(); // frontmatter 없음
        }
    } else {
        return input.clone(); // 빈 문자열
    }

    let lines = lines.skip(1);
    let mut in_frontmatter = true;
    let mut result = Vec::new();

    for line in lines {
        if in_frontmatter {
            let trimmed = line.trim();
            // frontmatter 끝나는 구분자도 '-'만 반복된 문자열
            if trimmed.chars().all(|c| c == '-') {
                in_frontmatter = false;
            }
            continue; // frontmatter 줄은 무조건 건너뜀
        } else {
            result.push(line);
        }
    }

    result.join("\n")
}