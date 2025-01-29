let cart = [];

async function getInital() {
  const data = await fetch(window.Shopify.routes.root + "cart.js", {
    headers: {
      "Content-Type": "application/json",
    },
  })
    .then((response) => {
      return response.json();
    })
    .catch((error) => {
      console.error("Error:", error);
    });
  cart = data.items;
}

getInital();

const addToCart = async (productId) => {
  const data = await fetch(window.Shopify.routes.root + "cart/add.js", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      items: [{ id: productId, quantity: 1 }],
    }),
  })
    .then((response) => {
      return response.json();
    })
    .catch((error) => {
      console.error("Error:", error);
    });

  cart = data.items;
};

const checkCartQuantity = async (productId) => {
  return cart.find((item) => item.id === productId)?.quantity;
};

export { addToCart, checkCartQuantity };
