function donePressed(ele_id) {
  var ele = document.getElementById(ele_id)
  ele.innerHTML = ""
}

function openEditDialog(obj, ele_id) {
  var attributes = Object.keys(obj)
  var ele = document.getElementById(ele_id)
  ele.innerHTML = ""

  for (var i = 0; i < attributes.length; i++) {
    ele.innerHTML += "<br>" + attributes[i] + " "
    ele.innerHTML += "<input value='" + obj[attributes[i]] + "' id='property-" + i + "'></input>"


  }

  ele.innerHTML += "<br><button id='done-button' onclick='donePressed(\"" + ele_id + "\")'>Done</button>"

  for (var i = 0; i < attributes.length; i++) {
    document.getElementById("property-" + i).addEventListener("change", function(event) {
      console.log(event)
      obj[attributes[i]] = ""
    });
  }
}
