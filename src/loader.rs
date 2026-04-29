use std::path::Path;

use lopdf::Document;

use crate::types::Chunk;

pub fn load_pdf(path: &Path) -> Result<Vec<Chunk>, Box<dyn std::error::Error>> {
    let doc = Document::load(path)?;
    let mut chunks = Vec::new();

    // 遍历页码（1-based）而不是索引
    for (page_num, _) in doc.get_pages() {
        let mut chunk = Chunk::new();
        chunk.metadata.page = Some(page_num); // 保存实际的页码
        chunk.content = doc.extract_text(&[page_num])?; // 使用 ? 而不是 unwrap
        chunks.push(chunk);
    }

    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile; // 在 Cargo.toml 的 [dev-dependencies] 中添加 tempfile = "3"

    use lopdf::{Object, Stream, content::Content, content::Operation, dictionary};

    fn create_test_pdf(texts: &[&str]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut doc = Document::with_version("1.5");

        // 共享字体资源
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        });
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! { "F1" => font_id },
        });

        let pages = doc.new_object_id();
        let mut kids = Vec::new();

        for (_, &page_text) in texts.iter().enumerate() {
            // 构建页面内容流
            let content = Content {
                operations: vec![
                    Operation::new("BT", vec![]),
                    Operation::new("Tf", vec!["F1".into(), 12.0.into()]),
                    Operation::new("Td", vec![50.0.into(), 750.0.into()]),
                    Operation::new("Tj", vec![Object::string_literal(page_text)]),
                    Operation::new("ET", vec![]),
                ],
            };
            let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));

            // 页面对象
            let page_id = doc.add_object(dictionary! {
                "Type" => "Page",
                "Parent" => pages,
                "Contents" => content_id,
                "Resources" => resources_id,
                "MediaBox" => vec![0.0.into(), 0.0.into(), 595.0.into(), 842.0.into()],
            });
            kids.push(page_id.into());
        }

        // 页面树
        let pages_dict = dictionary! {
            "Type" => "Pages",
            "Kids" => Object::Array(kids),
            "Count" => texts.len() as i64,
        };
        doc.objects.insert(pages, Object::Dictionary(pages_dict));

        // 目录
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages,
        });
        doc.trailer.set("Root", catalog_id);

        // 保存到内存 Vec<u8>
        let mut buffer = std::io::Cursor::new(Vec::new());
        doc.save_to(&mut buffer)?;
        Ok(buffer.into_inner())
    }
    fn create_test_pdf_data(texts: &[&str]) -> Vec<u8> {
        // 上面定义的 create_test_pdf 函数的包装，返回 Vec<u8>
        create_test_pdf(texts).unwrap()
    }

    #[test]
    fn test_load_pdf_with_generated_doc() {
        // 准备测试数据：两页，不同文本
        let expected_pages = vec![(1, "Hello, world!"), (2, "Rust is awesome.")];
        let pdf_data = create_test_pdf_data(&["Hello, world!", "Rust is awesome."]);

        // 将 PDF 数据写入临时文件
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&pdf_data).unwrap();
        let path = temp_file.path();

        // 调用被测试函数
        let chunks = load_pdf(path).unwrap();

        // 验证结果
        assert_eq!(chunks.len(), expected_pages.len());
        for (chunk, (page_num, expected_text)) in chunks.iter().zip(expected_pages.iter()) {
            assert_eq!(chunk.metadata.page, Some(*page_num));
            assert_eq!(chunk.content.trim(), expected_text.trim());
        }
    }
}
