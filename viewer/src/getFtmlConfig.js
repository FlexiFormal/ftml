const content = element.textContent;
try {
  const func = new Function("return (" + content + ")");
  return func();
} catch (e) {
  throw new Error("Failed to parse config object: " + e.message);
}
