use napi::bindgen_prelude::*;
use napi_derive::napi;
use tantivy as tv;
// Bring the trait into scope to use methods like `as_str()` on `OwnedValue`.
use tantivy::schema::Value;

/// Tantivy Snippet
///
/// Snippet contains a fragment of a document, and some highlighted
/// parts inside it.
#[napi]
pub struct Snippet {
  pub(crate) inner: tv::snippet::Snippet,
}

#[napi(object)]
pub struct Range {
  pub start: u32,
  pub end: u32,
}

#[napi]
impl Snippet {
  #[napi]
  pub fn to_html(&self) -> Result<String> {
    Ok(self.inner.to_html())
  }

  /// Returns highlighted ranges as **character offsets** (not byte offsets).
  /// Tantivy internally uses byte ranges on UTF-8 strings; this method
  /// converts them so they can be used directly with JS string.slice().
  #[napi]
  pub fn highlighted(&self) -> Vec<Range> {
    let fragment = self.inner.fragment();
    let highlighted = self.inner.highlighted();
    highlighted
      .iter()
      .map(|r| {
        let start_chars = fragment[..r.start].chars().count() as u32;
        let end_chars = fragment[..r.end].chars().count() as u32;
        Range {
          start: start_chars,
          end: end_chars,
        }
      })
      .collect()
  }

  #[napi]
  pub fn fragment(&self) -> Result<String> {
    Ok(self.inner.fragment().to_string())
  }
}

#[napi]
pub struct SnippetGenerator {
  pub(crate) field_name: String,
  pub(crate) inner: tv::snippet::SnippetGenerator,
}

#[napi]
impl SnippetGenerator {
  #[napi(factory)]
  pub fn create(
    searcher: &crate::Searcher,
    query: &crate::Query,
    schema: &crate::Schema,
    field_name: String,
  ) -> Result<SnippetGenerator> {
    let field = schema.inner.get_field(&field_name).map_err(|_| {
      Error::new(
        napi::Status::InvalidArg,
        format!("Field '{}' not found", field_name),
      )
    })?;
    let generator = tv::snippet::SnippetGenerator::create(&searcher.inner, query.get(), field)
      .map_err(|e| Error::new(napi::Status::GenericFailure, e.to_string()))?;

    Ok(SnippetGenerator {
      field_name,
      inner: generator,
    })
  }

  #[napi]
  pub fn snippet_from_doc(&self, doc: &crate::Document) -> crate::Snippet {
    let text: String = doc
      .iter_values_for_field(&self.field_name)
      .flat_map(|ov| ov.as_str())
      .collect::<Vec<&str>>()
      .join(" ");

    let result = self.inner.snippet(&text);
    Snippet { inner: result }
  }

  #[napi]
  pub fn set_max_num_chars(&mut self, max_num_chars: u32) {
    self.inner.set_max_num_chars(max_num_chars as usize);
  }
}
