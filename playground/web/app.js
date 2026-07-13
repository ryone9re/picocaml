import init, { run } from "./pkg/picocaml_playground.js";

const examples = [
  ["arithmetic", "let twice = fun x -> x * 2 in twice 21"],
  ["polymorphic identity", "let id = fun x -> x in id 7"],
  ["list", "match (1 :: (2 :: [])) with [] -> 0 | h :: t -> h"],
  ["type error", "1 + true"],
];

const source = document.querySelector("#source");
const type = document.querySelector("#type");
const value = document.querySelector("#value");
const ast = document.querySelector("#ast");
const status = document.querySelector("#status");
const errorCard = document.querySelector("#error-card");
const errorTitle = document.querySelector("#error-title");
const error = document.querySelector("#error");
const examplesPanel = document.querySelector("#examples");

function encode(value) {
  return btoa(unescape(encodeURIComponent(value)));
}

function decode(value) {
  return decodeURIComponent(escape(atob(value)));
}

function setOutput(element, text, empty = false) {
  element.textContent = text ?? "—";
  element.classList.toggle("empty", empty || !text);
}

function execute() {
  const program = source.value.trim();
  if (!program) return;
  status.textContent = "running";
  const report = JSON.parse(run(program));
  setOutput(type, report.ty, !report.ty);
  setOutput(value, report.value, !report.value);
  setOutput(ast, report.ast, !report.ast);
  errorCard.classList.toggle("hidden", !report.error);
  if (report.error) {
    errorTitle.textContent = `${report.phase} error`;
    error.textContent = report.error;
  }
  status.textContent = report.error ? "attention" : "done";
}

function loadFromHash() {
  if (!location.hash.startsWith("#code=")) return;
  try { source.value = decode(location.hash.slice(6)); } catch (_) { /* ignore malformed links */ }
}

document.querySelector("#run").addEventListener("click", execute);
source.addEventListener("keydown", (event) => {
  if ((event.metaKey || event.ctrlKey) && event.key === "Enter") execute();
  if (event.key === "Tab") { event.preventDefault(); document.execCommand("insertText", false, "  "); }
});
document.querySelector("#example").addEventListener("click", () => examplesPanel.classList.toggle("hidden"));
document.querySelector("#share").addEventListener("click", async (event) => {
  const url = `${location.origin}${location.pathname}#code=${encode(source.value)}`;
  history.replaceState(null, "", `#code=${encode(source.value)}`);
  await navigator.clipboard?.writeText(url);
  event.currentTarget.textContent = "Copied!";
  setTimeout(() => { event.currentTarget.textContent = "Copy link"; }, 1200);
});
examples.forEach(([name, code]) => {
  const button = document.createElement("button");
  button.textContent = name;
  button.addEventListener("click", () => { source.value = code; examplesPanel.classList.add("hidden"); execute(); });
  examplesPanel.append(button);
});

loadFromHash();
await init();
execute();
