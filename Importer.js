// Dependencies: Nodes.js
/* global Nodes */
/* global Node */
/* global Edges */
/* global Edge */

// Class used for importing exported json tree files.
//
// Primary method of interest is import()

class Importer { // eslint-disable-line no-unused-vars
  // Function used to validate and import an exported json object. See
  // exportTree() in Exporter.js for format structure.
  //
  // Returns: An array containing [Nodes, Edges] respectively
  import (jsonData, adtFormat = false) {
    const nodeStore = new Nodes()
    const edgeStore = new Edges()

    const modelUUID = jsonData.dataModel

    for (const node of jsonData.nodes) {
      const nodeID = node.id
      const nodeLabel = node.label
      const nodeAttributes = node.attributes
      let newNode = null

      if (modelUUID === Node.getUUID()) {
        newNode = new Node(nodeID, nodeLabel, nodeAttributes)
      } else if (modelUUID === EvitaNode.getUUID()) {
        newNode = new EvitaNode(nodeID, nodeLabel, nodeAttributes)
      }

      nodeStore.addNode(newNode)
    }

    for (const edge of jsonData.edges) {
      const newEdge = new Edge(edge.from, edge.to)
      edgeStore.addEdge(newEdge)
    }

    return [nodeStore, edgeStore, modelUUID]
  }
}
