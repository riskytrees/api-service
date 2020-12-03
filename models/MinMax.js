// Dependencies: Nodes.js
/* global Nodes */
/* global Node */

class MinMaxNode extends Node {
  constructor (id, label, attributeObj) {
    super(id, label, attributeObj)
  }

  static getUUID () {
    return '7df52155-0330-435b-b022-55586ec188a1'
  }

  modelLabelDisplay () {
    let effortStr = 'Unknown'

    if (this.attributes) {
      let effort = parseInt(this.attributes['effort'], 10)
      effortStr = effort.toString()
   }

    return this.label + ' | Effort: ' + effortStr
  }
}
