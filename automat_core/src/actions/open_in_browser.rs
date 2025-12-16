use crate::{Action, Result};

/// Action that opens a URL in a web browser.
///
/// By default, uses the system's default browser, but can be configured
/// to use a specific browser application.
///
/// ```no_run
/// // Use default browser
/// let action = OpenInBrowser::new("https://example.com");
///
/// // Use specific browser
/// let action = OpenInBrowser::new("https://example.com")
///     .with_browser("firefox");
/// ```
pub struct OpenInBrowser {
  url: String,
  browser: Option<String>,
}

impl OpenInBrowser {
  /// Creates a new action to open the given URL in the default browser.
  pub fn new(url: impl Into<String>) -> Self {
    Self {
      url: url.into(),
      browser: None,
    }
  }

  /// Specifies a particular browser to use instead of the default.
  pub fn with_browser(mut self, browser: impl Into<String>) -> Self {
    self.browser = Some(browser.into());
    self
  }

  /// Returns the URL that will be opened.
  pub fn url(&self) -> &str {
    &self.url
  }

  /// Returns the specific browser to use, if any.
  pub fn browser(&self) -> Option<&str> {
    self.browser.as_deref()
  }
}

impl Action for OpenInBrowser {
  fn run(&self) -> Result<()> {
    let result = if let Some(browser) = &self.browser {
      open::with(&self.url, browser)
    } else {
      open::that(&self.url)
    };

    result.map_err(Into::into)
  }
}
