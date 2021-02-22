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

  modelLabelDisplay (edgesStore, nodesStore) {
    let effortStr = '?'

    if (this.attributes && this.attributes.effort && this.getChildren(edgesStore, nodesStore).length === 0) {
      const effort = parseInt(this.attributes.effort, 10)
      effortStr = effort.toString()
    } else if (this.attributes && this.attributes.operator && this.getChildren(edgesStore, nodesStore).length > 0) {
      let effort = null

      if (this.attributes.operator === 'min' || this.attributes.operator === 'or') {
        // Min - Get easiest thing
        effort = this.getMinEffort(this.getChildren(edgesStore, nodesStore))
      } else if (this.attributes.operator === 'max' || this.attributes.operator === 'and') {
        // Max - Get hardest thing
        effort = this.getMaxEffort(this.getChildren(edgesStore, nodesStore))
      }

      if (effort) {
        effortStr = effort.toString()
      }
    }

    return this.label + '\nEffort: ' + effortStr
  }

  getMinEffort (nodes) {
    let lowestEffort = null

    for (const child of nodes) {
      if (child.attributes.effort) {
        const childEffort = parseInt(child.attributes.effort, 10)

        if (!lowestEffort || childEffort < lowestEffort) {
          lowestEffort = childEffort
        }
      }
    }

    return lowestEffort
  }

  getMaxEffort (nodes) {
    let highestEffort = null

    for (const child of nodes) {
      if (child.attributes.effort) {
        const childEffort = parseInt(child.attributes.effort, 10)

        if (!highestEffort || childEffort > highestEffort) {
          highestEffort = childEffort
        }
      }
    }

    return highestEffort
  }
}
