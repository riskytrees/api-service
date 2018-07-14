function donePressed(ele_id) {
  var ele = document.getElementById(ele_id)
  ele.innerHTML = ""
}

function openEditDialog(obj, ele_id) {
  var attributes = Object.keys(obj)
  var ele = document.getElementById(ele_id)
  ele.innerHTML = ""

  // Display existing attributes
  for (var i = 0; i < attributes.length; i++) {
    ele.innerHTML += "<br>" + attributes[i] + " "
    ele.innerHTML += "<input value='" + obj[attributes[i]] + "' id='property-" + i + "'></input>"
  }

  // UI for adding attributes
  ele.innerHTML += "<br><br>Add attribute:"
  ele.innerHTML += "<input id='new-attr-id-field' placeholder='New ID'></input> <input id='new-attr-val-field' placeholder='New Value'></input>"
  ele.innerHTML += " <button id='add-button'>Add Attribute</button>"

  ele.innerHTML += "<br><br><button id='done-button' onclick='donePressed(\"" + ele_id + "\")'>Done</button>"

  // Event listeners for editing attributes
  for (var i = 0; i < attributes.length; i++) {
    document.getElementById("property-" + i).addEventListener("change", function(event) {
      console.log(event)
      obj[attributes[event.target.id.split("-").pop()]] = document.getElementById(event.target.id).value
    });
  }

  document.getElementById("add-button").addEventListener("click", function(event) {
    console.log("Add clicked")
    // Save added attribute if one exists
    if (document.getElementById("new-attr-id-field").value !== "") {
      obj[document.getElementById("new-attr-id-field").value] = document.getElementById("new-attr-val-field").value
    }
  });
}
