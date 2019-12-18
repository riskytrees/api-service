function redrawHelper () {
  var data = {
    nodes: NodesStore.toVIS(),
    edges: EdgesStore.edges
  }

  globalNetwork.setData(data)
  globalNetwork.redraw()
}

function newNode (arg1, arg2, arg3) {
  if (ChosenModelUUID === Node.getUUID()) {
    return new Node(arg1, arg2, arg3)
  } else if (ChosenModelUUID === EvitaNode.getUUID()) {
    return new EvitaNode(arg1, arg2, arg3)
  }
}

function updateModel() {
  let oldNodeStore = NodesStore

  NodesStore = new Nodes()

  for (let node of oldNodeStore.nodes) {
    NodesStore.addNode(newNode(node.id, node.label, node.attributes))
  }

  console.log(ChosenModelUUID)
  document.getElementById('modelSelector').value = ChosenModelUUID

  redrawHelper()
}

function modelChanged () {
  let selector = document.getElementById('modelSelector')
  let chosenModel = selector.value

  ChosenModelUUID = chosenModel
  updateModel()
}

function populateModels () {
  let selector = document.getElementById('modelSelector')

  let defaultOption = document.createElement('option')
  defaultOption.value = Node.getUUID()
  defaultOption.textContent = 'Node'

  let evitaOption = document.createElement('option')
  evitaOption.value = EvitaNode.getUUID()
  evitaOption.textContent = 'EVITA'

  selector.appendChild(defaultOption)
  selector.appendChild(evitaOption)

  selector.onchange = function () { modelChanged() }
}

// Populate UI
populateModels()

// Attack Tree Setup
EdgesStore = new Edges()
NodesStore = new Nodes()
ChosenModelUUID = Node.getUUID()

// Create root node
globalRoot = newNode(0, 'Root Node', { 'root': true })
NodesStore.addNode(globalRoot)

// create a network
var container = document.getElementById('mynetwork')

// provide the data in the vis format
var data = {
  nodes: NodesStore.toVIS(),
  edges: EdgesStore.edges
}
var options = {
  layout: {
    hierarchical: {
      direction: 'UD',
      sortMethod: 'directed',
      nodeSpacing: 150,
      levelSeparation: 100
    }
  },
  interaction: { dragNodes: false },
  physics: {
    enabled: false
  }
}

// initialize your network!
globalNetwork = new vis.Network(container, data, options)

function addNode () {
  // Add a child.
  var nextID = NodesStore.generateUniqueNodeID()
  var child = newNode(nextID, 'Child Node ' + nextID, {})

  // Get selected Node
  var selectedNodes = globalNetwork.getSelectedNodes()

  if (selectedNodes === []) {
    return
  }

  NodesStore.getNode(selectedNodes[0]).addChild(child, EdgesStore, NodesStore)
  redrawHelper()
}

function editNode () {
  var selectedNodes = globalNetwork.getSelectedNodes()

  if (selectedNodes === []) {
    return
  }

  openEditDialog(NodesStore.getNode(selectedNodes[0]), 'editor', redrawHelper)
}

function exportTree () {
  let exporter = new Exporter()

  let exportJSON = exporter.exportTree(NodesStore, EdgesStore, false, ChosenModelUUID)

  let dataStr = 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(exportJSON))
  let anchor = document.getElementById('downloadAnchor')
  anchor.setAttribute('href', dataStr)
  anchor.setAttribute('download', 'attack_tree.json')
  anchor.click()
}

function importTree (event) {
  let importer = new Importer()

  let reader = new FileReader()
  reader.onload = function () {
    let importedTree = JSON.parse(reader.result)
    let [nodes, edges, dataModel] = importer.import(importedTree)
    console.log(nodes)
    console.log(edges)
    NodesStore = nodes
    EdgesStore = edges
    ChosenModelUUID = dataModel
    updateModel()

    redrawHelper()
  }
  reader.readAsText(event.target.files[0])
}
