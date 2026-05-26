use std::fs;
use std::path::Path;

use lopdf::{Object, Stream, content::Content, content::Operation, dictionary, Document};

fn create_test_pdf(texts: &[&str]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut doc = Document::with_version("1.5");

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

    for &page_text in texts {
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

        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.0.into(), 0.0.into(), 595.0.into(), 842.0.into()],
        });
        kids.push(page_id.into());
    }

    let pages_dict = dictionary! {
        "Type" => "Pages",
        "Kids" => Object::Array(kids),
        "Count" => texts.len() as i64,
    };
    doc.objects.insert(pages, Object::Dictionary(pages_dict));

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages,
    });
    doc.trailer.set("Root", catalog_id);

    let mut buffer = std::io::Cursor::new(Vec::new());
    doc.save_to(&mut buffer)?;
    Ok(buffer.into_inner())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pages = vec![
        "RAG Study Test Document - Page 1\n\nC++ smart pointers: shared_ptr uses reference counting to manage object lifetime. When the reference count reaches zero, memory is automatically freed. unique_ptr has exclusive ownership - it cannot be copied, only moved. weak_ptr breaks circular references between shared_ptrs; it doesn't increase the reference count and needs lock() to be promoted to shared_ptr before accessing the object.",
        "RAG Study Test Document - Page 2\n\nRust smart pointers: Box<T> is similar to unique_ptr, representing exclusive heap-allocated ownership. Rc<T> is a reference-counted shared pointer but only for single-threaded use. Arc<T> is an atomic reference-counted shared pointer that can be safely shared across threads. Weak<T> works with Rc/Arc to break cycles.",
        "RAG Study Test Document - Page 3\n\nBM25 is a ranking function based on the probabilistic retrieval model. It considers term frequency saturation and document length normalization. The k1 parameter controls term frequency saturation, and b controls document length normalization. Typical values: k1=1.5, b=0.75.\n\nEmbedding models map text to high-dimensional vector space. Semantically similar texts are closer together. Common distance metrics include cosine similarity and Euclidean distance. Cosine similarity is insensitive to vector length, making it more suitable for text semantic comparison.",
        "RAG Study Test Document - Page 4\n\nRAG (Retrieval-Augmented Generation) combines information retrieval and text generation. The first stage uses a retriever to find relevant document fragments from a knowledge base. The second stage feeds these fragments as context into a large language model, which generates answers based on the retrieved information.\n\nReranker is used for re-ranking in RAG systems. Top-K candidates (e.g., K=12) are retrieved first, then a more precise but slower re-ranking model re-scores them. The top N (e.g., N=3) are selected as the final context for the LLM.",
    ];

    let out_dir = Path::new("data");
    fs::create_dir_all(out_dir)?;
    let pdf_data = create_test_pdf(&pages)?;
    fs::write(out_dir.join("test.pdf"), &pdf_data)?;
    println!("Generated data/test.pdf with {} pages", pages.len());
    Ok(())
}
