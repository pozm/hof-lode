export interface poll {
    id: number;
    poll_name: string;
    open: boolean;
    close_date: string;
    type: number;
}
export interface poll_option_housing {
    address: string;
    image: string;
    name: string;
    option_id: number;
}