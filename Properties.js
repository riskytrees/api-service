
function openEditDialog(obj, ele) {
  var attributes = Object.keys(obj);
  ele.innerHTML = "";

  for (var i = 0; i < attributes.length; i++) {
    ele.innerHTML += "<br>" + attributes[i] + " ";
    ele.innerHTML += "<input value='" + obj[attributes[i]] + "'></input>"
  }
}
