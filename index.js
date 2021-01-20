function redrawHelper () {
  const data = {
    nodes: NodesStore.toVIS(EdgesStore),
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
  } else if (ChosenModelUUID === MinMaxNode.getUUID()) {
    return new MinMaxNode(arg1, arg2, arg3)
  }
}

function updateModel () {
  const oldNodeStore = NodesStore

  NodesStore = new Nodes()

  for (const node of oldNodeStore.nodes) {
    NodesStore.addNode(newNode(node.id, node.label, node.attributes))
  }

  console.log(ChosenModelUUID)
  document.getElementById('modelSelector').value = ChosenModelUUID

  redrawHelper()
}

function modelChanged () {
  const selector = document.getElementById('modelSelector')
  const chosenModel = selector.value

  ChosenModelUUID = chosenModel
  updateModel()
}

function populateModels () {
  const selector = document.getElementById('modelSelector')

  const defaultOption = document.createElement('option')
  defaultOption.value = Node.getUUID()
  defaultOption.textContent = 'Node'

  const evitaOption = document.createElement('option')
  evitaOption.value = EvitaNode.getUUID()
  evitaOption.textContent = 'EVITA'

  const minMaxOption = document.createElement('option')
  minMaxOption.value = MinMaxNode.getUUID()
  minMaxOption.textContent = 'MinMax'

  selector.appendChild(defaultOption)
  selector.appendChild(evitaOption)
  selector.appendChild(minMaxOption)

  selector.onchange = function () { modelChanged() }
}

// Populate UI
populateModels()

// Attack Tree Setup
EdgesStore = new Edges()
NodesStore = new Nodes()
ChosenModelUUID = Node.getUUID()

// Create root node
globalRoot = newNode(0, 'Root Node', { root: true })
NodesStore.addNode(globalRoot)

// create a network
const container = document.getElementById('mynetwork')

// provide the data in the vis format
const data = {
  nodes: NodesStore.toVIS(EdgesStore),
  edges: EdgesStore.edges
}
const options = {
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
  const nextID = NodesStore.generateUniqueNodeID()
  const child = newNode(nextID, 'Child Node ' + nextID, {})

  // Get selected Node
  const selectedNodes = globalNetwork.getSelectedNodes()

  if (selectedNodes === []) {
    return
  }

  NodesStore.getNode(selectedNodes[0]).addChild(child, EdgesStore, NodesStore)
  redrawHelper()
}

function editNode () {
  const selectedNodes = globalNetwork.getSelectedNodes()

  if (selectedNodes === []) {
    return
  }

  openEditDialog(NodesStore.getNode(selectedNodes[0]), 'editor', redrawHelper)
}

function exportTree () {
  const exporter = new Exporter()

  const exportJSON = exporter.exportTree(NodesStore, EdgesStore, false, ChosenModelUUID)

  const dataStr = 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(exportJSON))
  const anchor = document.getElementById('downloadAnchor')
  anchor.setAttribute('href', dataStr)
  anchor.setAttribute('download', 'attack_tree.json')
  anchor.click()
}

function importTree (event) {
  const importer = new Importer()

  const reader = new FileReader()
  reader.onload = function () {
    const importedTree = JSON.parse(reader.result)
    const [nodes, edges, dataModel] = importer.import(importedTree)
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
