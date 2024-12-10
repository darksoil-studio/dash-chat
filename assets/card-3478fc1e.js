var i=Object.defineProperty;var l=(r,o,t)=>o in r?i(r,o,{enumerable:!0,configurable:!0,writable:!0,value:t}):r[o]=t;var e=(r,o,t)=>(l(r,typeof o!="symbol"?o+"":o,t),t);import{s as a,x as d,i as s}from"./lit-element-e618a7a4.js";class n extends a{render(){return d`
      <slot name="title"></slot>
      <slot></slot>
      <slot name="cta"></slot>
    `}}e(n,"styles",[s`
      :host {
        margin-bottom: 5rem;
        padding: 3rem 1rem;
        border: 1px solid #9fa3ae;
        border-radius: 20px;
        background-color: #ffffff;
        text-align: center;
        position: relative;
        display: flex;
        flex-direction: column;
        min-width: 0;
        word-wrap: break-word;
        background-color: #fff;
        background-clip: border-box;
        min-width: 20ch;
      }
    `]);export{n as RocketCard};
