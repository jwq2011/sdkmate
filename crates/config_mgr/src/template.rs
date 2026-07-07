use anyhow::Result;
use std::collections::HashMap;

/// 模板渲染器
pub struct TemplateRenderer {
    variables: HashMap<String, String>,
}

impl TemplateRenderer {
    /// 创建新的模板渲染器
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// 添加变量
    pub fn add_variable(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    /// 批量添加变量
    pub fn add_variables(&mut self, variables: HashMap<String, String>) {
        self.variables.extend(variables);
    }

    /// 渲染模板（替换 {{VAR}} 占位符）
    pub fn render(&self, template: &str) -> String {
        let mut result = template.to_string();
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }

    /// 渲染文件
    pub fn render_file(&self, path: &str) -> Result<String> {
        let content = std::fs::read_to_string(path)?;
        Ok(self.render(&content))
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple() {
        let mut renderer = TemplateRenderer::new();
        renderer.add_variable("NAME".to_string(), "World".to_string());

        let result = renderer.render("Hello, {{NAME}}!");
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_render_multiple() {
        let mut renderer = TemplateRenderer::new();
        renderer.add_variable("GREETING".to_string(), "Hello".to_string());
        renderer.add_variable("NAME".to_string(), "World".to_string());

        let result = renderer.render("{{GREETING}}, {{NAME}}!");
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_render_no_placeholder() {
        let renderer = TemplateRenderer::new();
        let result = renderer.render("No placeholders here");
        assert_eq!(result, "No placeholders here");
    }
}
