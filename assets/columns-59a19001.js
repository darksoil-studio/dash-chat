var i=Object.defineProperty;var n=(t,e,o)=>e in t?i(t,e,{enumerable:!0,configurable:!0,writable:!0,value:o}):t[e]=o;var r=(t,e,o)=>(n(t,typeof e!="symbol"?e+"":e,o),o);import{s as l,i as s,x as a}from"./lit-element-e618a7a4.js";class c extends l{render(){return a` <slot id="content"></slot> `}}r(c,"styles",[s`
      #content {
        display: flex;
        gap: var(--space-m, 2rem);
        flex-direction: column;
        flex-wrap: wrap;
      }
      ::slotted(*) {
        flex: 1;
        height: auto;
      }

      @media (min-width: 600px) {
        #content {
          flex-direction: row;
        }
      }
    `]);export{c as RocketColumns};
