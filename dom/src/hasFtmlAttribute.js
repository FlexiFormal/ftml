if (typeof window === "undefined") {
  return false;
}
if (node.tagName.toLowerCase() === "img") {
  // replace "srv:" by server url
  const attributes = node.attributes;
  for (let i = 0; i < attributes.length; i++) {
    if (attributes[i].name === "data-flams-src") {
      const src = attributes[i].value;
      node.setAttribute("src", src.replace("srv:", window.FLAMS_SERVER_URL));
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
