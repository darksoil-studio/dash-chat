var i=Object.defineProperty;var o=(e,t,n)=>t in e?i(e,t,{enumerable:!0,configurable:!0,writable:!0,value:n}):e[t]=n;var a=(e,t,n)=>(o(e,typeof t!="symbol"?t+"":t,n),n);import{s as r,x as d,i as c}from"./lit-element-e618a7a4.js";class s extends r{render(){return d`
      <div id="rocket-content-area">
        <slot></slot>
      </div>
    `}}a(s,"styles",[c`
      :host {
        display: block;
        padding: var(--rocket-content-area-padding, 0);
      }

      :host([no-padding]) {
        padding: 0;
      }

      /** rocket-content-area */
      #rocket-content-area {
        padding: 0 20px;
        display: block;
        justify-content: space-between;
        align-items: center;
      }
      @media screen and (min-width: 1024px) {
        #rocket-content-area {
          max-width: 1200px;
          margin: 0 auto;
          position: relative;
        }
      }
    `]);export{s as RocketContentArea};
