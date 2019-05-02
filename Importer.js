// Dependencies: Nodes.js

// Class used for importing exported json tree files.
//
// Primary method of interest is import()

class Importer {
  constructor () {

  }

  // Function used to validate and import an exported json object. See
  // exportTree() in Exporter.js for format structure.
  //
  // Returns: An array containing [Nodes, Edges] respectively
  import(jsonData, adtFormat = false) {
    let nodeStore = new Nodes()
    let edgeStore = new Edges()

    let nodeIDCtr = 0
    for (let node of jsonData.nodes) {
      let nodeID = node.id
      let nodeLabel = node.label
      let nodeAttributes = node.attributes


      let newNode = new Node(nodeID, nodeLabel, nodeAttributes)
      nodeStore.addNode(newNode)
    }

    for (let edge of jsonData.edges) {
      let newEdge = new Edge(edge.from, edge.to)
      edgeStore.addEdge(newEdge)
    }

    return [nodeStore, edgeStore]
  }
}
