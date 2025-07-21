//- Generated from Product.rs

export interface Product {


    /**
* 商品ID（ユニークな識別子）
*/
    product_id: string;

    /**
* 商品の価格（小数対応）
*/
    price: number;

    /**
* 在庫数（単位数）
*/
    quantity: number;


}


//- Generated from User.rs

export interface User {


    /**
* ユーザーID（ユニークな識別子）
*/
    id: number;

    /**
* ユーザーの名前
*/
    name: string;

    /**
* ユーザーのメールアドレス（オプション）
*/
    email: string | undefined;


}


