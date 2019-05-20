// Dependencies: Nodes.js
/* global Nodes */
/* global Node */

class EvitaNode extends Node {
  constructor (id, label, attributeObj) {
    super(id, label, attributeObj)
  }

  modelLabelDisplay() {
    return this.label + " | EVITA"
  }
}
