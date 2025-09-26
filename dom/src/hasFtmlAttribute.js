if (typeof window === "undefined") {
  return false;
}
if (node.tagName.toLowerCase() === "img") {
  // replace "srv:" by server url
  const attributes = node.attributes;
  for (let i = 0; i < attributes.length; i++) {
    if (attributes[i].name === "data-ftml-src") {
      const src = attributes[i].value;
      if (window.FTML_SERVER_URL === undefined) {
          window.FTML_SERVER_URL = "";
      }
      node.setAttribute("src", src.replace("srv:", window.FTML_SERVER_URL));
        node.attributes.removeNamedItem("data-ftml-src");
      break;
    }
  }
}
//if (node.tagName.toLowerCase() === "section") {return true}
const attributes = node.attributes;
for (let i = 0; i < attributes.length; i++) {
  if (attributes[i].name.startsWith("data-ftml-")) {
    return true;
  }
}
return false;
