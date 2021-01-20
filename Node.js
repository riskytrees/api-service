// This is not node.js, but rather a "node" object used in the attack trees.

// This is an especially important class, because it can be overriden in order to implement other attack models.
// By default it does simple min/max costing with one attribute.
// Data stored in a d3-compatible format: https://github.com/d3/d3-hierarchy/blob/master/README.md#hierarchy
class Node { // eslint-disable-line no-unused-vars
  constructor (id, label, attributeObj) {
    this.attributes = attributeObj
    this.label = label
    this.id = id
  }

  addChild (node, edgesStore, nodesStore) {
    // Add to node store.
    nodesStore.addNode(node)

    // Add edge from this.id to node.id
    const newEdge = new Edge(this.id, node.id)
    edgesStore.addEdge(newEdge)
  }

  getChildren (edgesStore, nodesStore) {
    const children = []

    for (const anEdge of edgesStore.edges) {
      if (anEdge.from === this.id) {
        children.push(nodesStore.getNode(anEdge.to))
      }
    }

    return children
  }

  editLabel (newLabel) {
    this.label = newLabel
  }

  // Overridable Functions

  static getUUID () {
    return '43daa996-8208-499f-be1b-f6c34c84d9df'
  }

  // Returns a string representing what should be displayed as the title of a
  // node on an attack tree.
  modelLabelDisplay (edgesStore, nodesStore) {
    return this.label
  }
}

class Nodes { // eslint-disable-line no-unused-vars
  constructor () {
    this.nodes = []
  }

  toVIS (edgesStore) {
    const visData = []

    for (const aNode of this.nodes) {
      const copyNode = new Node(aNode.id, aNode.label, aNode.attributes)
      copyNode.label = aNode.modelLabelDisplay(edgesStore, this)
      visData.push(copyNode)
    }

    return visData
  }

  addNode (node) {
    this.nodes.push(node)
  }

  generateUniqueNodeID () {
    // Sort nodes. Then, find a gap.
    this.nodes.sort(function (a, b) {
      return a.id - b.id
    })

    for (let i = 0; i < this.nodes.length; i++) {
      if (i == 0) {
        if (this.nodes[i].id != 0) {
          return 0
        }
      } else {
        if (this.nodes[i].id - this.nodes[i - 1].id != 1) {
          return this.nodes[i].id + 1
        }
      }
    }

    return this.nodes.length
  }

  getNode (id) {
    for (let i = 0; i < this.nodes.length; i++) {
      if (this.nodes[i].id == id) {
        return this.nodes[i]
      }
    }

    return null
  }
}

class Edge { // eslint-disable-line no-unused-vars
  constructor (from, to) {
    this.from = from
    this.to = to
  }
}

class Edges { // eslint-disable-line no-unused-vars
  constructor () {
    this.edges = []
  }

  addEdge (edge) {
    this.edges.push(edge)
  }
}
