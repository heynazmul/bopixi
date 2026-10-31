/* Pre-rename install path for the design-sync snippet. Pages saved with
   <script src=".../doop-sync.js?key=..."> still load this file; it forwards
   to the current snippet with the same query and data-* options, so those
   pages keep capturing without editing their HTML. */
;(function () {
  'use strict'
  if (window.bopixiSync || window.doopSync) return
  var script = document.currentScript
  if (!script || !script.src) return
  var tag = document.createElement('script')
  tag.async = true
  tag.src = script.src.replace(/\/doop-sync\.js(\?|$)/, '/bopixi-sync.js$1')
  if (tag.src === script.src) return
  var options = ['data-key', 'data-host', 'data-mask']
  for (var i = 0; i < options.length; i++) {
    var value = script.getAttribute(options[i])
    if (value !== null) tag.setAttribute(options[i], value)
  }
  ;(document.head || document.documentElement).appendChild(tag)
})()
