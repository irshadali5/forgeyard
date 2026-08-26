// Initialize Mermaid diagrams in mdBook documentation
document.addEventListener("DOMContentLoaded", function () {
  if (typeof mermaid === "undefined") {
    return;
  }

  // Detect current mdBook theme
  var htmlClass = document.querySelector("html").className;
  var isDark = htmlClass.indexOf("navy") !== -1 || htmlClass.indexOf("coal") !== -1 || htmlClass.indexOf("ayu") !== -1;

  mermaid.initialize({
    startOnLoad: false,
    theme: isDark ? "dark" : "default",
    securityLevel: "loose",
    themeVariables: {
      fontFamily: "Inter, system-ui, -apple-system, sans-serif",
      fontSize: "14px",
      primaryColor: isDark ? "#2b3b55" : "#e0e7ff",
      primaryTextColor: isDark ? "#f1f5f9" : "#1e293b",
      primaryBorderColor: isDark ? "#3b82f6" : "#6366f1",
      lineColor: isDark ? "#60a5fa" : "#4f46e5",
      secondaryColor: isDark ? "#1e293b" : "#f8fafc",
      tertiaryColor: isDark ? "#0f172a" : "#ffffff"
    }
  });

  // Convert markdown code blocks with class `language-mermaid` or `mermaid` to mermaid divs
  var codeBlocks = document.querySelectorAll("pre > code.language-mermaid, pre > code.mermaid");
  codeBlocks.forEach(function (codeBlock) {
    var pre = codeBlock.parentNode;
    var container = document.createElement("div");
    container.className = "mermaid";
    container.textContent = codeBlock.textContent;
    pre.parentNode.replaceChild(container, pre);
  });

  mermaid.run();
});
