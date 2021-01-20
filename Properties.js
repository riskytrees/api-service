function donePressed (ele_id) {
  const ele = document.getElementById(ele_id)
  ele.innerHTML = ''
}

function openEditDialog (obj, ele_id, redrawFunc) {
  const attributes = Object.keys(obj.attributes)
  const ele = document.getElementById(ele_id)
  ele.innerHTML = ''

  // Display name
  ele.innerHTML += "<br><input value='" + obj.label + "' id='node-label'></input>"

  // Display existing attributes
  for (let i = 0; i < attributes.length; i++) {
    ele.innerHTML += '<br><br>' + attributes[i] + ' '
    ele.innerHTML += "<input value='" + obj.attributes[attributes[i]] + "' id='property-" + i + "'></input>"
  }

  // UI for adding attributes
  ele.innerHTML += '<br><br>Add attribute: '
  ele.innerHTML += "<input id='new-attr-id-field' placeholder='New ID'></input> <input id='new-attr-val-field' placeholder='New Value'></input>"
  ele.innerHTML += " <button id='add-button'>Add Attribute</button>"

  ele.innerHTML += "<br><br><button id='done-button' onclick='donePressed(\"" + ele_id + "\")'>Done</button>"

  // Event listeners for editing attributes
  for (let i = 0; i < attributes.length; i++) {
    document.getElementById('property-' + i).addEventListener('change', function (event) {
      console.log(event)
      obj.attributes[attributes[event.target.id.split('-').pop()]] = document.getElementById(event.target.id).value
    })
  }

  document.getElementById('node-label').addEventListener('change', function (event) {
    console.log(event)
    obj.label = document.getElementById(event.target.id).value
    redrawFunc()
  })

  document.getElementById('add-button').addEventListener('click', function (event) {
    console.log('Add clicked')
    // Save added attribute if one exists
    if (document.getElementById('new-attr-id-field').value !== '') {
      obj.attributes[document.getElementById('new-attr-id-field').value] = document.getElementById('new-attr-val-field').value
      donePressed(ele_id)
      openEditDialog(obj, ele_id, redrawFunc)
      redrawFunc()
    }
  })
}
