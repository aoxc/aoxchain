document.addEventListener("DOMContentLoaded", () => {
  const btn = document.getElementById("preview-btn");
  const terminal = document.getElementById("terminal");

  if (btn && terminal) {
    btn.addEventListener("click", () => {
      terminal.textContent += "\n[PREVIEW] Canonical command preview is visible.";
    });
  }
});
