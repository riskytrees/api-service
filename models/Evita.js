// Dependencies: Nodes.js
/* global Nodes */
/* global Node */

class EvitaNode extends Node {
  constructor (id, label, attributeObj) {
    super(id, label, attributeObj)
  }

  static getUUID () {
    return '4f483a97-0b3c-4755-83b0-085f674b6d94'
  }

  modelLabelDisplay (edgesStore, nodesStore) {
    return this.label + ' | EVITA'
  }
}
