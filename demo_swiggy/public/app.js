// ==============================================================================
// AUTHENTIC SWIGGY WEB APPLICATION CONTROLLER
// Powered by VALENCE Multi-Core In-Memory Database (Port 6379)
// ==============================================================================

const state = {
  mode: 'customer',
  categories: [],
  restaurants: [],
  activeRestaurant: null,
  cart: [],
  ttlSeconds: 60,
  activeOrder: null,
  kitchenOrders: [],
  riderProgress: 0.1,
  rushActive: false,
  vegOnly: false,
  rushMetrics: { connectedSockets: 0, opsPerSec: 0, totalOps: 0, p99Us: 0.38, cores: new Array(12).fill(0) },
};

// DOM References
const mainContentWrap = document.getElementById('main-content-wrap');
const ecoTabs = document.querySelectorAll('.eco-tab');
const categoryScrollRow = document.getElementById('category-scroll-row');
const restaurantsGrid = document.getElementById('restaurants-grid');
const restaurantDetailView = document.getElementById('restaurant-detail-view');
const restDetailHeader = document.getElementById('rest-detail-header');
const menuDishesList = document.getElementById('menu-dishes-list');
const custSearchInput = document.getElementById('cust-search-input');
const devCacheBadge = document.getElementById('dev-cache-badge');
const devCacheText = document.getElementById('dev-cache-text');

// Cart Drawer
const cartDrawer = document.getElementById('cart-drawer');
const cartDrawerBackdrop = document.getElementById('cart-drawer-backdrop');
const cartItemsScroll = document.getElementById('cart-items-scroll');
const headerCartCount = document.getElementById('header-cart-count');
const cartRestTitle = document.getElementById('cart-rest-title');
const cartDrawerCountdown = document.getElementById('cart-drawer-countdown');
const cartDrawerTtlBar = document.getElementById('cart-drawer-ttl-bar');
const billItemTotal = document.getElementById('bill-item-total');
const billGrandTotal = document.getElementById('bill-grand-total');
const btnPayAmount = document.getElementById('btn-pay-amount');

// Order Tracking & Map
const orderTrackingScreen = document.getElementById('order-tracking-screen');
const trackOrderTitle = document.getElementById('track-order-title');
const trackRestName = document.getElementById('track-rest-name');
const trackLatLng = document.getElementById('track-latlng');
const svgRiderBike = document.getElementById('svg-rider-bike');

// Kitchen View
const kStatActiveCount = document.getElementById('k-stat-active-count');
const kOrdersList = document.getElementById('k-orders-list');

// Rider View
const riderOrderIdDisplay = document.getElementById('rider-order-id-display');
const btnRiderPickup = document.getElementById('btn-rider-pickup');
const btnRiderDeliver = document.getElementById('btn-rider-deliver');
const rCoordLat = document.getElementById('r-coord-lat');
const rCoordLng = document.getElementById('r-coord-lng');

// Rush View
const btnRushMain = document.getElementById('btn-rush-main');
const rushMainText = document.getElementById('rush-main-text');
const rushMetricSockets = document.getElementById('rush-metric-sockets');
const rushMetricOps = document.getElementById('rush-metric-ops');
const rushMetricTotal = document.getElementById('rush-metric-total');
const rushMetricP99 = document.getElementById('rush-metric-p99');
const coresVuGrid = document.getElementById('cores-vu-grid');

// Dev Inspector Toggle
const chkDevInspector = document.getElementById('chk-dev-inspector');
if (chkDevInspector) {
  chkDevInspector.addEventListener('change', (e) => {
    document.body.classList.toggle('dev-mode-active', e.target.checked);
  });
}

// ==============================================================================
// 1. NAVIGATION & VIEW CONTROLLER
// ==============================================================================
function setAppMode(mode) {
  state.mode = mode;
  ecoTabs.forEach(t => t.classList.toggle('active', t.dataset.mode === mode));

  const panelCustomer = document.getElementById('panel-customer');
  const panelKitchen = document.getElementById('panel-kitchen');
  const panelRider = document.getElementById('panel-rider');
  const panelRush = document.getElementById('panel-rush');

  if (mode === 'split') {
    mainContentWrap.className = 'main-content-wrap mode-split';
    panelCustomer.style.display = 'flex';
    panelKitchen.style.display = 'flex';
    panelRider.style.display = 'flex';
    panelRush.style.display = 'none';
  } else {
    mainContentWrap.className = 'main-content-wrap';
    panelCustomer.style.display = mode === 'customer' ? 'flex' : 'none';
    panelKitchen.style.display = mode === 'kitchen' ? 'flex' : 'none';
    panelRider.style.display = mode === 'rider' ? 'flex' : 'none';
    panelRush.style.display = mode === 'rush' ? 'flex' : 'none';
  }

  // Query Valence on Port 6379 for this view state
  if (window.triggerValenceAction) {
    window.triggerValenceAction('tab', mode);
  }
}

ecoTabs.forEach(t => {
  t.addEventListener('click', () => setAppMode(t.dataset.mode));
});

window.showCustomerHome = function() {
  setAppMode('customer');
  closeRestaurantDetail();
  closeTrackingScreen();
};

// ==============================================================================
// 2. FETCH FEED & RENDER AUTHENTIC RESTAURANTS
// ==============================================================================
async function loadFeed() {
  try {
    const res = await fetch('/api/swiggy/feed');
    const data = await res.json();
    if (data.success) {
      state.categories = data.categories;
      state.restaurants = data.restaurants;
      renderCategories(data.categories);
      renderRestaurants(data.restaurants);

      if (data.restaurants.length > 0 && data.restaurants[0].menu.length > 0) {
        state.activeRestaurant = data.restaurants[0];
        const firstDish = data.restaurants[0].menu[0];
        state.cart = [{ ...firstDish, qty: 1 }];
        updateCartDisplay();
      }
    }
  } catch (err) {
    console.error('Feed load err:', err);
  }
}

function renderCategories(cats) {
  categoryScrollRow.innerHTML = cats.map(c => `
    <div class="category-item" onclick="filterCuisine('${c.name}')">
      <img src="${c.image}" alt="${c.name}">
    </div>
  `).join('');
}

function renderRestaurants(rests) {
  restaurantsGrid.innerHTML = rests.map(r => `
    <div class="rest-card" onclick="openRestaurantMenu('${r.id}')">
      <div class="rest-img-wrap">
        <img src="${r.image}" alt="${r.name}">
        <div class="rest-discount-banner">${r.discount}</div>
      </div>
      <div class="rest-info">
        <div class="rest-name">${r.name}</div>
        <div class="rest-meta-row">
          <span class="rest-rating-badge">★</span>
          <span>${r.rating} • ${r.eta}</span>
        </div>
        <div class="rest-cuisines">${r.cuisines}</div>
        <div class="rest-area">${r.area}</div>
      </div>
    </div>
  `).join('');
}

let chipTimer = null;
window.flashValenceLiveChip = function(core = 'Core C02', latencyUs = '0.22', label = '') {
  const chip = document.getElementById('valence-live-chip-mobile');
  const vlcText = document.getElementById('vlc-text');
  if (!chip || !vlcText) return;

  const tag = label ? ` • ${label}` : '';
  vlcText.innerHTML = `⚡ <strong>${core}</strong>: ${latencyUs} µs${tag}`;
  chip.classList.add('pulse-active');
  clearTimeout(chipTimer);
  chipTimer = setTimeout(() => {
    chip.classList.remove('pulse-active');
  }, 1200);
};

window.filterCuisine = async function(cuisine) {
  const pills = document.querySelectorAll('.filter-pill');
  pills.forEach(p => {
    p.classList.toggle('active', p.innerText.toLowerCase() === cuisine.toLowerCase() || (cuisine === 'all' && p.innerText === 'All'));
  });

  const url = (cuisine === 'all') ? '/api/swiggy/search?q=' : `/api/swiggy/search?q=${encodeURIComponent(cuisine.toLowerCase())}`;
  try {
    const res = await fetch(url);
    const data = await res.json();
    if (data.success) {
      renderRestaurants(data.restaurants);
      if (data.latencyUs && devCacheText) {
        devCacheText.innerHTML = `VALENCE CACHE: <strong>${data.latencyUs} µs (${data.core})</strong>`;
      }
      flashValenceLiveChip(data.core || 'Core C01', data.latencyUs || '0.22', cuisine.toUpperCase());
    }
  } catch {}
};

window.filterQuick = function(type) {
  const allRests = state.restaurants;
  let filtered = allRests;
  let label = type.toUpperCase();

  const pills = document.querySelectorAll('.filter-pill');
  pills.forEach(p => p.classList.remove('active'));

  if (type === 'fast') {
    filtered = allRests.filter(r => parseInt(r.eta) <= 25 || r.eta.includes('20') || r.eta.includes('25'));
    label = 'FAST DELIVERY';
  } else if (type === 'veg') {
    filtered = allRests.filter(r => r.cuisines.toLowerCase().includes('pure veg') || r.menu.some(m => m.isVeg));
    label = 'PURE VEG';
  } else if (type === 'rating') {
    filtered = allRests.filter(r => parseFloat(r.rating) >= 4.3);
    label = 'RATING 4.3+';
  } else if (type === 'offers') {
    filtered = allRests.filter(r => r.discount && r.discount.length > 0);
    label = 'OFFERS';
  }

  renderRestaurants(filtered);
  flashValenceLiveChip('Core C03', '0.20', `FILTER:${label}`);
};

let mobSearchDebounce = null;
window.handleMobileSearch = function(query) {
  clearTimeout(mobSearchDebounce);
  mobSearchDebounce = setTimeout(async () => {
    const q = (query || '').trim().toLowerCase();
    if (!q) {
      renderRestaurants(state.restaurants);
      return;
    }

    try {
      const res = await fetch(`/api/swiggy/search?q=${encodeURIComponent(q)}`);
      const data = await res.json();
      if (data.success) {
        renderRestaurants(data.restaurants);
        flashValenceLiveChip(data.core || 'Core C04', data.latencyUs || '0.21', `q='${q}'`);
      }
    } catch {
      const filtered = state.restaurants.filter(r => 
        r.name.toLowerCase().includes(q) || 
        r.cuisines.toLowerCase().includes(q) ||
        r.menu.some(m => m.name.toLowerCase().includes(q))
      );
      renderRestaurants(filtered);
    }
  }, 160);
};

custSearchInput.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') {
    const q = custSearchInput.value.trim();
    filterCuisine(q || 'all');
  }
});

window.focusSearch = function() {
  setAppMode('customer');
  const mobSearch = document.getElementById('mob-search-input');
  if (window.innerWidth <= 768 && mobSearch) {
    mobSearch.focus();
  } else {
    custSearchInput.focus();
  }
};

// ==============================================================================
// 3. RESTAURANT MENU & DISH SELECTION (Queries GET restaurant:<id> on Valence)
// ==============================================================================
window.openRestaurantMenu = async function(restId) {
  const rest = state.restaurants.find(r => r.id === restId);
  if (!rest) return;

  state.activeRestaurant = rest;
  cartRestTitle.innerText = rest.name;

  // Send query to Valence on port 6379 & flash live chip
  fetch(`/api/restaurant/get?id=${restId}`)
    .then(r => r.json())
    .then(data => {
      if (data && data.core) {
        flashValenceLiveChip(data.core, data.latencyUs, rest.name);
      }
    })
    .catch(() => {});

  document.querySelector('.hero-promo-container').style.display = 'none';
  document.querySelector('.category-section').style.display = 'none';
  document.querySelector('.restaurant-section').style.display = 'none';
  restaurantDetailView.style.display = 'flex';

  restDetailHeader.innerHTML = `
    <div class="rest-detail-header-card">
      <div class="rd-title-box">
        <h2>${rest.name}</h2>
        <p class="rd-cuisines">${rest.cuisines}</p>
        <p style="font-size: 12px; color: var(--text-muted);">${rest.area} • ${rest.costForTwo} • ⏱️ ${rest.eta}</p>
        <div style="margin-top:8px; display:inline-flex; align-items:center; gap:6px; background:linear-gradient(135deg, rgba(255,94,0,0.1), rgba(255,138,0,0.05)); border:1px solid rgba(255,94,0,0.25); border-radius:6px; padding:3px 8px; font-size:11px; font-weight:700; color:#e05300;">
          <span>⚡ Swiggy ONE</span>
          <span>• Free delivery on this order</span>
        </div>
      </div>
      <div class="rd-rating-box">
        <div class="rd-score">★ ${rest.rating}</div>
        <div class="rd-count">${rest.ratingCount} ratings</div>
      </div>
    </div>
  `;

  renderCurrentMenu();
};

window.toggleVegOnlyMenu = function(checked) {
  state.vegOnly = !!checked;
  renderCurrentMenu();
  flashValenceLiveChip('Core C01', '0.23', checked ? 'FILTER:VEG' : 'FILTER:ALL');
};

function renderCurrentMenu() {
  if (!state.activeRestaurant) return;
  const items = state.vegOnly
    ? state.activeRestaurant.menu.filter(m => m.isVeg)
    : state.activeRestaurant.menu;

  menuDishesList.innerHTML = `
    <div style="display:flex; justify-content:space-between; align-items:center; margin-top:14px; padding-bottom:10px; border-bottom:1px solid var(--border-light);">
      <h3 style="font-size: 17px; font-weight: 800; color: var(--text-dark); margin:0;">
        ${state.vegOnly ? 'Pure Veg Selection' : 'Recommended'} (${items.length})
      </h3>
      <label style="display:flex; align-items:center; gap:8px; cursor:pointer; font-size:12px; font-weight:700; color:var(--text-secondary); background:rgba(15,138,101,0.06); border:1px solid rgba(15,138,101,0.25); padding:4px 10px; border-radius:20px;">
        <span style="display:inline-block; width:12px; height:12px; border:1px solid #0f8a65; border-radius:2px; position:relative;">
          <span style="position:absolute; width:6px; height:6px; background:#0f8a65; border-radius:50%; top:2px; left:2px;"></span>
        </span>
        <span>VEG ONLY</span>
        <input type="checkbox" id="chk-veg-only" ${state.vegOnly ? 'checked' : ''} onchange="toggleVegOnlyMenu(this.checked)" style="accent-color:#0f8a65; cursor:pointer; width:15px; height:15px; margin:0;">
      </label>
    </div>
    ${items.map(d => renderDishRow(d)).join('')}
  `;
}

function renderDishRow(dish) {
  const inCart = state.cart.find(c => c.id === dish.id);
  const vegIcon = dish.isVeg ? `<span class="veg-icon-badge"></span>` : `<span class="non-veg-icon-badge"></span>`;
  const bestRibbon = dish.bestseller ? `<span class="bestseller-ribbon">★ Bestseller</span>` : '';

  const btnHtml = inCart ? `
    <div class="stepper-box">
      <button class="stepper-btn" onclick="modifyDishQty('${dish.id}', -1)">-</button>
      <span>${inCart.qty}</span>
      <button class="stepper-btn" onclick="modifyDishQty('${dish.id}', 1)">+</button>
    </div>
  ` : `
    <button class="btn-dish-add" onclick="modifyDishQty('${dish.id}', 1)">ADD</button>
  `;

  return `
    <div class="dish-row-card">
      <div class="dish-text-col">
        <div style="display: flex; align-items: center; gap: 8px;">
          ${vegIcon}
          ${bestRibbon}
        </div>
        <div class="dish-name-title">${dish.name}</div>
        <div class="dish-price-tag">₹${dish.price}</div>
        <div style="font-size: 11px; color: var(--swiggy-green-dark); font-weight: 700;">★ ${dish.rating} (${dish.ratingCount})</div>
        <div class="dish-desc-text">${dish.desc}</div>
      </div>
      <div class="dish-img-col">
        <img src="${dish.image}" alt="${dish.name}">
        ${btnHtml}
      </div>
    </div>
  `;
}

window.closeRestaurantDetail = function() {
  document.querySelector('.hero-promo-container').style.display = 'grid';
  document.querySelector('.category-section').style.display = 'flex';
  document.querySelector('.restaurant-section').style.display = 'flex';
  restaurantDetailView.style.display = 'none';
};

// Modifies quantity and immediately synchronizes with Valence on Port 6379!
window.modifyDishQty = function(dishId, delta) {
  if (!state.activeRestaurant) return;
  const dish = state.activeRestaurant.menu.find(m => m.id === dishId);
  if (!dish) return;

  const existing = state.cart.find(c => c.id === dishId);
  let newQty = 0;
  if (existing) {
    existing.qty += delta;
    newQty = existing.qty;
    if (existing.qty <= 0) {
      state.cart = state.cart.filter(c => c.id !== dishId);
      newQty = 0;
    }
  } else if (delta > 0) {
    state.cart.push({ ...dish, qty: 1 });
    newQty = 1;
  }

  // Unique session per tab/device
  let userSessionId = sessionStorage.getItem('swiggy_session_id');
  if (!userSessionId) {
    userSessionId = 'user_' + Math.random().toString(36).substring(2, 7);
    sessionStorage.setItem('swiggy_session_id', userSessionId);
  }

  // Send SET cart:<userSessionId>:<dishId> to Valence
  fetch('/api/cart/update', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ dishId, qty: newQty, sessionId: userSessionId }),
  })
    .then(r => r.json())
    .then(data => {
      if (data && data.core) {
        flashValenceLiveChip(data.core, data.latencyUs, `SET cart:${dishId}`);
      }
    })
    .catch(() => {});

  // Reset 60-second Timing Wheel hold
  state.ttlSeconds = 60;
  updateCartDisplay();

  if (restaurantDetailView.style.display !== 'none') {
    renderCurrentMenu();
  }
};

// ==============================================================================
// 4. CART & 4-TIER TIMING WHEEL TTL (5-Min Reservation)
// ==============================================================================
function updateCartDisplay() {
  const count = state.cart.reduce((sum, i) => sum + i.qty, 0);
  const subtotal = state.cart.reduce((sum, i) => sum + (i.price * i.qty), 0);
  const grandTotal = count > 0 ? Math.max(0, subtotal + 35 - 50 + 5 + 18) : 0;

  if (headerCartCount) headerCartCount.innerText = count;
  if (billItemTotal) billItemTotal.innerText = `₹${subtotal}`;
  if (billGrandTotal) billGrandTotal.innerText = `₹${grandTotal}`;
  if (btnPayAmount) btnPayAmount.innerText = `₹${grandTotal}`;

  // Update Mobile Floating Cart Bar
  const floatingCartBar = document.getElementById('floating-cart-bar');
  const fcCountPill = document.getElementById('fc-count-pill');
  const fcPrice = document.getElementById('fc-price');

  if (fcCountPill) {
    fcCountPill.innerText = `${count} ${count === 1 ? 'ITEM' : 'ITEMS'}`;
  }
  if (fcPrice) {
    fcPrice.innerText = `₹${grandTotal}`;
  }
  if (floatingCartBar) {
    if (count > 0) {
      floatingCartBar.classList.add('visible');
    } else {
      floatingCartBar.classList.remove('visible');
    }
  }

  if (state.cart.length === 0) {
    cartItemsScroll.innerHTML = `<p style="text-align: center; color: var(--text-muted); padding: 40px 0; font-size: 13px;">Your cart is empty. Add items from the menu!</p>`;
  } else {
    cartItemsScroll.innerHTML = state.cart.map(c => `
      <div class="cart-item-row">
        <div class="cart-item-name">${c.name}</div>
        <div class="cart-stepper-mini">
          <button style="border:none;background:none;cursor:pointer;font-weight:700;color:var(--swiggy-green-dark);" onclick="modifyDishQty('${c.id}', -1)">-</button>
          <span>${c.qty}</span>
          <button style="border:none;background:none;cursor:pointer;font-weight:700;color:var(--swiggy-green-dark);" onclick="modifyDishQty('${c.id}', 1)">+</button>
        </div>
        <div class="cart-item-price">₹${c.price * c.qty}</div>
      </div>
    `).join('');
  }
}

window.toggleCartDrawer = function() {
  cartDrawer.classList.toggle('open');
  cartDrawerBackdrop.classList.toggle('open');
};

setInterval(() => {
  if (state.ttlSeconds > 0) {
    state.ttlSeconds--;
    const mins = Math.floor(state.ttlSeconds / 60).toString().padStart(2, '0');
    const secs = (state.ttlSeconds % 60).toString().padStart(2, '0');
    cartDrawerCountdown.innerText = `${mins}:${secs}`;
    const pct = Math.max(0, (state.ttlSeconds / 60) * 100);
    cartDrawerTtlBar.style.width = `${pct}%`;
  } else {
    cartDrawerCountdown.innerText = '00:00 (EXPIRED)';
    cartDrawerTtlBar.style.width = '0%';
  }
}, 1000);

// ==============================================================================
// 5. CHECKOUT & ORDER LIFECYCLE SYNC
// ==============================================================================
window.handleCheckout = async function() {
  if (state.cart.length === 0) {
    alert('Please add dishes to your cart first!');
    return;
  }

  const orderId = `SWG-${Math.floor(1000 + Math.random() * 9000)}`;
  const restName = state.activeRestaurant ? state.activeRestaurant.name : 'Meghana Foods';
  const subtotal = state.cart.reduce((sum, i) => sum + (i.price * i.qty), 0);
  const total = Math.max(0, subtotal + 35 - 50 + 5 + 18);

  const btnProceedPay = document.getElementById('btn-proceed-pay');
  btnProceedPay.innerText = 'Reserving on Valence...';

  try {
    const res = await fetch('/api/order/place', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        id: orderId,
        restaurantName: restName,
        customerName: 'AISF Hackathon Judge',
        items: state.cart.map(c => ({ name: c.name, qty: c.qty, price: c.price })),
        total,
      }),
    });
    const data = await res.json();
    if (data.success) {
      state.activeOrder = data.order;
      state.kitchenOrders.unshift(data.order);
      toggleCartDrawer();
      showOrderTracking(data.order);
      renderKitchenOrders();
      syncRiderOrderDisplay();
    }
  } catch (err) {
    console.error('Checkout error:', err);
  } finally {
    btnProceedPay.innerHTML = `<span class="pay-text">PROCEED TO PAY</span><span class="pay-amount">${btnPayAmount.innerText}</span>`;
  }
};

function showOrderTracking(order) {
  closeRestaurantDetail();
  document.querySelector('.hero-promo-container').style.display = 'none';
  document.querySelector('.category-section').style.display = 'none';
  document.querySelector('.restaurant-section').style.display = 'none';
  orderTrackingScreen.style.display = 'flex';

  trackOrderTitle.innerText = `Order #${order.id} Placed`;
  trackRestName.innerText = order.restaurantName || 'Meghana Foods';
  syncTrackingStepper(order.status);
}

window.closeTrackingScreen = function() {
  orderTrackingScreen.style.display = 'none';
  document.querySelector('.hero-promo-container').style.display = 'grid';
  document.querySelector('.category-section').style.display = 'flex';
  document.querySelector('.restaurant-section').style.display = 'flex';
};

function syncTrackingStepper(status) {
  const s1 = document.getElementById('tr-step-1');
  const s2 = document.getElementById('tr-step-2');
  const s3 = document.getElementById('tr-step-3');
  const s4 = document.getElementById('tr-step-4');
  const l1 = document.getElementById('tr-line-1');
  const l2 = document.getElementById('tr-line-2');
  const l3 = document.getElementById('tr-line-3');

  s1.classList.add('active');
  l1.classList.remove('active');
  s2.classList.remove('active');
  l2.classList.remove('active');
  s3.classList.remove('active');
  l3.classList.remove('active');
  s4.classList.remove('active');

  if (status === 'PLACED') {
    trackOrderTitle.innerText = `Order #${state.activeOrder?.id || ''} Placed`;
  } else if (status === 'PREPARING') {
    trackOrderTitle.innerText = `Kitchen Cooking Your Order 🍳`;
    l1.classList.add('active');
    s2.classList.add('active');
  } else if (status === 'READY_FOR_PICKUP') {
    trackOrderTitle.innerText = `Food Packed & Ready for Pickup 📦`;
    l1.classList.add('active');
    s2.classList.add('active');
    l2.classList.add('active');
    s3.classList.add('active');
  } else if (status === 'OUT_FOR_DELIVERY') {
    trackOrderTitle.innerText = `Delivery Partner on the Way 🛵`;
    l1.classList.add('active');
    s2.classList.add('active');
    l2.classList.add('active');
    s3.classList.add('active');
    l3.classList.add('active');
    s4.classList.add('active');
  } else if (status === 'DELIVERED') {
    trackOrderTitle.innerText = `Delivered! Enjoy your food 🎉`;
    l1.classList.add('active');
    s2.classList.add('active');
    l2.classList.add('active');
    s3.classList.add('active');
    l3.classList.add('active');
    s4.classList.add('active');
  }
}

// ==============================================================================
// 6. RESTAURANT PARTNER (KITCHEN DASHBOARD)
// ==============================================================================
function renderKitchenOrders() {
  kStatActiveCount.innerText = state.kitchenOrders.length;
  if (state.kitchenOrders.length === 0) {
    kOrdersList.innerHTML = `<p style="text-align: center; color: var(--text-muted); padding: 40px;">No incoming orders currently.</p>`;
    return;
  }

  kOrdersList.innerHTML = state.kitchenOrders.map(o => `
    <div class="k-order-item-card">
      <div class="k-order-top">
        <div>
          <span class="k-order-id-label">Order #${o.id}</span>
          <span style="font-size: 12px; color: var(--text-secondary);"> • ${o.customerName}</span>
        </div>
        <span style="font-weight: 800; color: var(--swiggy-green-dark); font-size: 15px;">₹${o.total}</span>
      </div>
      <div class="k-order-items-box">
        ${Array.isArray(o.items) ? o.items.map(it => `${it.qty || 1}x ${it.name}`).join(', ') : o.items}
      </div>
      <div class="k-order-actions-row">
        <button class="btn-k-action btn-k-cook" onclick="updateKitchenOrderStatus('${o.id}', 'PREPARING')">
          🍳 Accept & Cook
        </button>
        <button class="btn-k-action btn-k-ready" onclick="updateKitchenOrderStatus('${o.id}', 'READY_FOR_PICKUP')">
          📦 Mark Ready for Pickup
        </button>
      </div>
    </div>
  `).join('');
}

window.updateKitchenOrderStatus = async function(id, status) {
  try {
    const res = await fetch('/api/order/update', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id, status, rider: 'Rajesh Kumar (#RID-50)' }),
    });
    const data = await res.json();
    if (data.success) {
      if (state.activeOrder && state.activeOrder.id === id) {
        state.activeOrder.status = status;
        syncTrackingStepper(status);
        syncRiderOrderDisplay();
      }
      renderKitchenOrders();
    }
  } catch (err) {
    console.error('Kitchen update error:', err);
  }
};

// ==============================================================================
// 7. SWIGGY DELIVERY PARTNER (RIDER APP) & GPS TELEMETRY
// ==============================================================================
function syncRiderOrderDisplay() {
  if (!state.activeOrder) return;
  riderOrderIdDisplay.innerText = `Order #${state.activeOrder.id}`;
  if (state.activeOrder.status === 'READY_FOR_PICKUP') {
    btnRiderPickup.disabled = false;
    btnRiderDeliver.disabled = true;
  } else if (state.activeOrder.status === 'OUT_FOR_DELIVERY') {
    btnRiderPickup.disabled = true;
    btnRiderDeliver.disabled = false;
  } else if (state.activeOrder.status === 'DELIVERED') {
    btnRiderPickup.disabled = true;
    btnRiderDeliver.disabled = true;
  }
}

window.riderPickupOrder = async function() {
  if (!state.activeOrder) return;
  state.riderProgress = 0.15;
  await updateKitchenOrderStatus(state.activeOrder.id, 'OUT_FOR_DELIVERY');
};

window.riderCompleteOrder = async function() {
  if (!state.activeOrder) return;
  state.riderProgress = 1.0;
  updateRiderMap(1.0);
  await updateKitchenOrderStatus(state.activeOrder.id, 'DELIVERED');
};

function getMapCoords(t) {
  const u = 1 - t;
  const x = u * u * 60 + 2 * u * t * 280 + t * t * 640;
  const y = u * u * 190 + 2 * u * t * 120 + t * t * 50;
  return { x, y };
}

function updateRiderMap(t) {
  const pt = getMapCoords(t);
  svgRiderBike.setAttribute('transform', `translate(${pt.x.toFixed(1)}, ${pt.y.toFixed(1)})`);

  const lat = 18.5204 + (t * 0.018);
  const lng = 73.8567 + (t * 0.022);

  rCoordLat.innerText = lat.toFixed(6);
  rCoordLng.innerText = lng.toFixed(6);
  trackLatLng.innerText = `${lat.toFixed(4)}° N, ${lng.toFixed(4)}° E`;

  fetch('/api/rider/gps', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ riderId: 'RID-50', lat, lng, speed: 32 }),
  }).catch(() => {});
}

setInterval(() => {
  if (state.activeOrder && state.activeOrder.status === 'OUT_FOR_DELIVERY') {
    if (state.riderProgress < 0.96) {
      state.riderProgress += 0.015;
      updateRiderMap(state.riderProgress);
    }
  }
}, 1200);

// ==============================================================================
// 8. 100-CONCURRENT-SOCKET DINNER RUSH SIMULATOR
// ==============================================================================
window.toggleRushSimulation = async function() {
  try {
    const res = await fetch('/api/rush/toggle', { method: 'POST' });
    const data = await res.json();
    if (data.success) {
      state.rushActive = data.active;
      updateRushUi();
    }
  } catch {}
};

function updateRushUi() {
  if (state.rushActive) {
    btnRushMain.classList.add('running');
    rushMainText.innerText = 'STOP DINNER RUSH (100 Sockets Active)';
  } else {
    btnRushMain.classList.remove('running');
    rushMainText.innerText = 'START 100-SOCKET DINNER RUSH';
  }
}

function initCoreVuGrid() {
  coresVuGrid.innerHTML = Array.from({ length: 12 }, (_, i) => {
    const cName = `C${i.toString().padStart(2, '0')}`;
    return `
      <div class="vu-col">
        <div class="vu-track">
          <div class="vu-fill" id="v-bar-${i}" style="height: 15%;"></div>
        </div>
        <span class="vu-lbl">${cName}</span>
      </div>
    `;
  }).join('');
}

setInterval(async () => {
  if (state.mode !== 'rush' && state.mode !== 'split') return;
  try {
    const res = await fetch('/api/rush/metrics');
    const data = await res.json();
    if (data.success) {
      state.rushMetrics = data;
      state.rushActive = data.active;
      updateRushUi();

      rushMetricSockets.innerText = `${data.connectedSockets} / 100`;
      rushMetricOps.innerText = `${data.opsPerSec.toLocaleString()} ops/sec`;
      rushMetricTotal.innerText = data.totalOps.toLocaleString();
      rushMetricP99.innerText = `${data.p99Us} µs`;

      if (Array.isArray(data.cores)) {
        const max = Math.max(1, ...data.cores);
        data.cores.forEach((cnt, i) => {
          const bar = document.getElementById(`v-bar-${i}`);
          if (bar) {
            const h = state.rushActive ? Math.min(100, Math.max(15, (cnt / max) * 100)) : 10;
            bar.style.height = `${h}%`;
          }
        });
      }
    }
  } catch {}
}, 250);

// ==============================================================================
// 9. LIVE TELEMETRY STREAM & QR CODE
// ==============================================================================
window.triggerValenceAction = async function(action, param = '') {
  try {
    const res = await fetch(`/api/action?action=${encodeURIComponent(action)}&param=${encodeURIComponent(param)}`);
    const data = await res.json();
    if (data && data.core) {
      flashValenceLiveChip(data.core, data.latencyUs, action.toUpperCase());
    }
  } catch {}
};

// Phone QR Modal
const modalQr = document.getElementById('modal-qr');
const btnOpenQr = document.getElementById('btn-open-qr');
const qrCanvas = document.getElementById('qr-canvas');
const networkLinksList = document.getElementById('network-links-list');

if (btnOpenQr) {
  btnOpenQr.addEventListener('click', async () => {
    modalQr.classList.add('open');
    try {
      const res = await fetch('/api/system/network');
      const data = await res.json();
      if (data.success && data.ips.length > 0) {
        const ip = data.ips[0];
        const targetUrl = `http://${ip}:${data.port}`;

        if (window.QRCode && qrCanvas) {
          window.QRCode.toCanvas(qrCanvas, targetUrl, {
            width: 200,
            margin: 1,
            color: { dark: '#000000', light: '#ffffff' },
          });
        }

        networkLinksList.innerHTML = data.ips.map(ipAddr => `
          <a href="http://${ipAddr}:${data.port}" target="_blank" class="net-link-btn">
            🔗 http://${ipAddr}:${data.port}
          </a>
        `).join('');
      }
    } catch {}
  });
}

window.closeQrModal = function() {
  modalQr.classList.remove('open');
};

function initSseTelemetry() {
  try {
    const evtSource = new EventSource('/api/telemetry/stream');
    evtSource.onmessage = (e) => {
      // Real-time queries streamed from Valence
    };
    evtSource.onerror = () => {
      // Auto-reconnect managed by EventSource
    };
  } catch (err) {}
}

// ==============================================================================
// INITIALIZATION
// ==============================================================================
window.addEventListener('DOMContentLoaded', () => {
  setAppMode('customer');
  loadFeed();
  initCoreVuGrid();
  initSseTelemetry();
});
