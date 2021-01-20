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

      if (this.attributes && this.attributes.effort) {
         const effort = parseInt(this.attributes.effort, 10)
         effortStr = effort.toString()
      } else if (this.getChildren(edgesStore, nodesStore).length > 0) {
         // Min - Get easiest thing
         let lowestEffort = null;

         for (const child of this.getChildren(edgesStore, nodesStore)) {
            if (child.attributes.effort) {
               const childEffort = parseInt(child.attributes.effort, 10)

               if (!lowestEffort || childEffort < lowestEffort) {
                  lowestEffort = childEffort
               }
            }

         }

         if (lowestEffort) {
            effortStr = lowestEffort.toString()
         }
      }

      return this.label + '\nEffort: ' + effortStr
   }
}
