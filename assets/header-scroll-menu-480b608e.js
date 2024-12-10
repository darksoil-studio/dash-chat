var r=Object.defineProperty;var a=(t,e,n)=>e in t?r(t,e,{enumerable:!0,configurable:!0,writable:!0,value:n}):t[e]=n;var i=(t,e,n)=>(a(t,typeof e!="symbol"?e+"":e,n),n);import{s as o,x as d,i as s}from"./lit-element-e618a7a4.js";import{RocketContentArea as c}from"./content-area-368b6216.js";customElements.get("rocket-content-area")||customElements.define("rocket-content-area",c);class h extends o{render(){return d`
      <div class="header-inner">
        <rocket-content-area no-padding>
          <div class="header-logo"></div>
          <nav class="header-navigation">
            <slot id="content"></slot>
          </nav>
        </rocket-content-area>
      </div>
    `}}i(h,"styles",[s`
      :host {
        display: flex;
        align-items: center;
        position: sticky;
        height: 100px;
        top: -50px; /* height - .header-inner height => 100-50 = 50 */
        z-index: 100;
      }

      .header-inner {
        height: 50px;
        position: sticky;
        top: 0;
        width: 100%;
        padding-right: var(--rocket-header-scroll-menu-padding-right, 0);
        text-align: var(--rocket-header-scroll-menu-text-align, center);
      }

      ::slotted(a) {
        text-decoration: none;
        padding: 0.25rem;
        line-height: 50px;
      }

      @media screen and (min-width: 1024px) {
        ::slotted(a) {
          padding: 0.75rem;
        }
      }
    `]);export{h as RocketHeaderScrollMenu};
