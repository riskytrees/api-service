// Class used for handling exportation of attack trees.
//
// The de-facto exportation method here would be to simply json-dump the two
// arrays. Unfortunately, that will not work adequately, as a requirement
// for the exporation/importation implementation is that is should be
// comformant to the format used by ADTool2.
//
// This means we need to construct an XML-based tree in a manner consistent
// with that tool. Unfortunately, ADTool2 never defined a formal spec, so we
// must reverse engineer it slightly.
//
// That said, we also support a "nicer" format that actually is formally
// defined. The hope is that, over time, ADTool2's usage will become redundant
// and this format will replace it.
class Exporter {
  constructor () {

  }

  // Function used for exporting a tree.
  //
  // Standard Export Format:
  //
  // exportedData = {
  //   nodes: [
  //     {
  //       id: Int(ID),
  //       label: String("Node Title"),
  //       attributes: [{}, ...]
  //     }, ...
  //   ],
  //   edges: [
  //     {
  //       to: Int(NodeID),
  //       from: Int(OtherNodeID)
  //     }
  //   ]
  // }
  //
  //
  // Returns: A json object containing the tree data if adtFormat is false.
  //          A string representation of the adt XML data if adtFormat is true.
  exportTree (nodeStore, edgeStore, adtFormat = false) {
    let nodes = []
    let edges = []

    for (let node of nodeStore.nodes) {
      nodes.push({
        'id': node.id,
        'label': node.label,
        'attributes': node.attributes
      })
    }

    for (let edge of edgeStore.edges) {
      edges.push({
        'from': edge.from,
        'to': edge.to
      })
    }

    let output = {
      'nodes': nodes,
      'edges': edges
    }

    return output
  }
}
