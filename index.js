function redrawHelper () {
  var data = {
    nodes: NodesStore.nodes,
    edges: EdgesStore.edges
  }

  globalNetwork.setData(data)
  globalNetwork.redraw()
}

// Attack Tree Setup
EdgesStore = new Edges()
NodesStore = new Nodes()

// Create root node
globalRoot = new Node(0, 'Root Node', { 'root': true })
NodesStore.addNode(globalRoot)

// create a network
var container = document.getElementById('mynetwork')

// provide the data in the vis format
var data = {
  nodes: NodesStore.nodes,
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
  var child = new Node(nextID, 'Child Node ' + nextID, {})

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

  let exportJSON = exporter.exportTree(NodesStore, EdgesStore)

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
    let [nodes, edges] = importer.import(importedTree)
    console.log(nodes)
    console.log(edges)
    NodesStore = nodes
    EdgesStore = edges
    redrawHelper()
  }
  reader.readAsText(event.target.files[0])
}
