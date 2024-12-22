

pub trait GameLogic {
    type CustomServerEvent: Serialize + DeserializeOwned + Debug + Clone;
    type CustomClientEvent: Serialize + DeserializeOwned + Debug + Clone;

    
}